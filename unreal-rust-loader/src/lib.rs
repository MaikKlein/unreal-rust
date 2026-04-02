use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use libloading::Library;

use unreal_ffi::{RegisterUnrealBindings, RustBindings, UnrealBindings};

#[cfg(target_os = "windows")]
const PLUGIN_EXTENSION: &str = "dll";
#[cfg(target_os = "linux")]
const PLUGIN_EXTENSION: &str = "so";
#[cfg(target_os = "macos")]
const PLUGIN_EXTENSION: &str = "dylib";

#[cfg(target_os = "windows")]
const PLUGIN_LIB_NAME: &str = "unreal_rust_example.dll";
#[cfg(target_os = "linux")]
const PLUGIN_LIB_NAME: &str = "libunreal_rust_example.so";
#[cfg(target_os = "macos")]
const PLUGIN_LIB_NAME: &str = "libunreal_rust_example.dylib";

/// Find the loader dylib's own filesystem path at runtime using platform APIs.
/// This lets us locate other files relative to the UE project Binaries directory.
fn find_own_dylib_path() -> Option<PathBuf> {
    #[cfg(unix)]
    {
        unsafe extern "C" {
            fn dladdr(addr: *const u8, info: *mut libc::Dl_info) -> libc::c_int;
        }
        let mut info: libc::Dl_info = unsafe { std::mem::zeroed() };
        let result =
            unsafe { dladdr(find_own_dylib_path as *const u8, &mut info) };
        if result != 0 && !info.dli_fname.is_null() {
            let path = unsafe { std::ffi::CStr::from_ptr(info.dli_fname) };
            path.to_str().ok().map(PathBuf::from)
        } else {
            None
        }
    }
    #[cfg(windows)]
    {
        None // Windows uses hardcoded paths in the original code
    }
}

/// Resolve the path to the example plugin library.
///
/// Search order:
/// 1. UNREAL_RUST_TARGET_DIR env var (absolute path to cargo target dir)
/// 2. Relative to the loader dylib: traverse up from Binaries/ to find
///    the workspace target/ directory (handles the symlinked plugin layout)
/// 3. Fallback to cargo default target/release/
fn resolve_plugin_path() -> PathBuf {
    // 1. Explicit env var
    if let Ok(dir) = std::env::var("UNREAL_RUST_TARGET_DIR") {
        return PathBuf::from(dir).join(PLUGIN_LIB_NAME);
    }

    // 2. Derive from loader's own location
    // Loader lives at: <project>/Binaries/unreal_rust_loader.dylib
    // Workspace root is: <project>/../../  (through the Plugins/RustPlugin symlink)
    // Cargo output is: <workspace>/target/release/
    if let Some(own_path) = find_own_dylib_path() {
        if let Some(binaries_dir) = own_path.parent() {
            // Try: <binaries>/../../../target/release/ (project -> workspace via setup.sh layout)
            let workspace_root = binaries_dir.join("../../..");
            for profile in ["release", "debug", "development"] {
                let candidate = workspace_root.join("target").join(profile).join(PLUGIN_LIB_NAME);
                if candidate.exists() {
                    return candidate;
                }
            }
        }
    }

    // 3. Fallback
    PathBuf::from("target/release").join(PLUGIN_LIB_NAME)
}

pub struct Plugin {
    library: Library,
    register_unreal_bindings: RegisterUnrealBindings,
    timestamp: SystemTime,
}
impl Plugin {
    pub fn new(
        unreal_bindings: &UnrealBindings,
        rust_bindings: &mut RustBindings,
        path: &Path,
    ) -> Result<Self, String> {
        let library = unsafe {
            Library::new(path).map_err(|e| format!("Failed to load {:?}: {}", path, e))?
        };

        let metadata = std::fs::metadata(path)
            .map_err(|e| format!("Failed to read metadata for {:?}: {}", path, e))?;

        let register_unreal_bindings: RegisterUnrealBindings = unsafe {
            *library
                .get::<RegisterUnrealBindings>("register_unreal_bindings\0")
                .map_err(|e| format!("Failed to find register_unreal_bindings in {:?}: {}", path, e))?
        };

        (register_unreal_bindings)(unreal_bindings.clone(), rust_bindings);

        Ok(Plugin {
            library,
            register_unreal_bindings,
            timestamp: metadata.modified().unwrap(),
        })
    }
}

static mut LOADER: *mut Loader = std::ptr::null_mut();

pub struct Loader {
    path: PathBuf,
    loaded_plugin: Option<Plugin>,
    unreal_bindings: UnrealBindings,
    hotreload_id: u32,
}
impl Loader {
    pub fn new(unreal_bindings: UnrealBindings, path: PathBuf) -> Self {
        Loader {
            path,
            loaded_plugin: None,
            unreal_bindings,
            hotreload_id: 0,
        }
    }

    pub fn is_out_of_date(&self) -> bool {
        let Ok(metadata) = std::fs::metadata(&self.path) else {
            return false;
        };
        let Ok(timestamp) = metadata.modified() else {
            return false;
        };

        let Some(plugin) = self.loaded_plugin.as_ref() else {
            return false;
        };

        timestamp > plugin.timestamp
    }

    pub fn load(&mut self, rust_bindings: &mut RustBindings) {
        // Unload the currently loaded plugin
        self.loaded_plugin = None;

        if !self.path.exists() {
            eprintln!(
                "[unreal-rust] Plugin not found at {:?}, skipping load",
                self.path
            );
            return;
        }

        let root_dir = self
            .path
            .parent()
            .map(|p| p.join("rust"))
            .unwrap_or_else(|| PathBuf::from("Binaries/rust"));

        for entry in root_dir.read_dir().into_iter().flatten().flatten() {
            if entry.path().is_dir()
                && let Some(file_name) = entry.file_name().to_str()
                && file_name.starts_with("rust-plugin")
            {
                // Try to delete every hot reload folder. Some we can't delete because the debugger
                // might keep them open
                let _ = fs::remove_dir_all(entry.path());
            }
        }

        let _ = fs::create_dir_all(&root_dir);

        let hot_reload_dir = root_dir.join(format!("rust-plugin-{}", self.hotreload_id));
        if hot_reload_dir.exists() {
            let _ = fs::remove_dir_all(&hot_reload_dir);
        }
        let hotreload_lib_name = format!("rust_plugin.{}", PLUGIN_EXTENSION);
        let hotreload_lib_path = hot_reload_dir.join(&hotreload_lib_name);
        let _ = fs::create_dir_all(&hot_reload_dir);

        if let Err(e) = fs::copy(&self.path, &hotreload_lib_path) {
            eprintln!(
                "[unreal-rust] Failed to copy {:?} to {:?}: {}",
                self.path, hotreload_lib_path, e
            );
            return;
        }

        // On macOS, copied dylibs lose their code signature and the OS will
        // kill the process with SIGKILL (Code Signature Invalid). Re-sign
        // with an ad-hoc signature so the hot-reloaded copy can be loaded.
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("codesign")
                .args(["--force", "--sign", "-"])
                .arg(&hotreload_lib_path)
                .output();
        }

        // Copy debug symbols on platforms that use sidecar files
        #[cfg(target_os = "windows")]
        if let Some(pdb_dir) = self.path.parent() {
            let pdb_path = pdb_dir.join("unreal_rust_example.pdb");
            if pdb_path.exists() {
                let _ = fs::copy(&pdb_path, hot_reload_dir.join("unreal_rust_example.pdb"));
            }
        }

        match Plugin::new(&self.unreal_bindings, rust_bindings, &hotreload_lib_path) {
            Ok(plugin) => {
                eprintln!("[unreal-rust] Loaded plugin from {:?}", self.path);
                self.loaded_plugin = Some(plugin);
                self.hotreload_id += 1;
            }
            Err(e) => {
                eprintln!("[unreal-rust] {}", e);
            }
        }
    }
}

#[unsafe(no_mangle)]
extern "C" fn register_unreal_bindings(bindings: UnrealBindings) -> u32 {
    unsafe {
        if LOADER.is_null() {
            let plugin_path = resolve_plugin_path();
            eprintln!(
                "[unreal-rust] Resolved plugin path: {:?} (exists: {})",
                plugin_path,
                plugin_path.exists()
            );
            LOADER = Box::leak(Box::new(Loader::new(bindings, plugin_path))) as *mut _;
        }
    }
    1
}

#[unsafe(no_mangle)]
unsafe extern "C" fn is_out_of_date() -> u32 {
    unsafe {
        if !LOADER.is_null() {
            (*LOADER).is_out_of_date() as u32
        } else {
            0
        }
    }
}
#[unsafe(no_mangle)]
unsafe extern "C" fn try_load(rust_bindings: *mut RustBindings) -> u32 {
    unsafe {
        if !LOADER.is_null() {
            (*LOADER).load(&mut *rust_bindings);
            1
        } else {
            0
        }
    }
}
