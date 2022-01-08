use bevy_ecs::schedule::Schedule;
use unreal_reflect::registry::ReflectionRegistry;

use crate::ffi::UnrealBindings;

pub trait UnrealModule {
    fn initialize() -> Self;
    fn register(registry: &mut ReflectionRegistry);
    fn systems(startup: &mut Schedule, update: &mut Schedule);
}
pub static mut BINDINGS: Option<UnrealBindings> = None;

#[macro_export]
macro_rules! implement_unreal_module {
    ($module: ty) => {
        static mut MODULE: Option<$crate::core::UnrealCore<$module>> = None;
        #[no_mangle]
        pub unsafe extern "C" fn register_unreal_bindings(bindings: $crate::ffi::UnrealBindings) -> $crate::ffi::RustBindings {
            $crate::module::BINDINGS = Some(bindings);
            let _ = $crate::log::init();
            let core = $crate::core::UnrealCore::new();

            MODULE = Some(core);
            $crate::ffi::RustBindings {
                retrieve_uuids: retrieve_uuids,
            }
        }
        #[no_mangle]
        pub unsafe extern "C" fn retrieve_uuids(ptr: *mut ffi::Uuid, len: *mut usize) {
            $crate::core::UnrealCore::retrieve_uuids(MODULE.as_mut().unwrap(), ptr, len);
        }

        #[no_mangle]
        pub extern "C" fn tick(dt: f32) -> $crate::ffi::ResultCode {
            let r = std::panic::catch_unwind(|| unsafe {
                $crate::core::UnrealCore::tick(MODULE.as_mut().unwrap(), dt);
            });
            match r {
                Ok(_) => $crate::ffi::ResultCode::Success,
                Err(_) => $crate::ffi::ResultCode::Panic,
            }
        }

        #[no_mangle]
        pub extern "C" fn begin_play() -> $crate::ffi::ResultCode {
            let r = std::panic::catch_unwind(|| unsafe {
                $crate::core::UnrealCore::begin_play(MODULE.as_mut().unwrap());
            });
            match r {
                Ok(_) => $crate::ffi::ResultCode::Success,
                Err(_) => $crate::ffi::ResultCode::Panic,
            }
        }
    };
}

pub fn bindings() -> &'static UnrealBindings {
    unsafe { BINDINGS.as_ref().unwrap() }
}
