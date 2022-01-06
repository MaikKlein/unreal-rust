use crate::ffi::UnrealBindings;

pub trait UnrealModule {
    fn initialize() -> Self;
    fn begin_play(&mut self) {}
    fn tick(&mut self, dt: f32) {}
}
pub static mut BINDINGS: Option<UnrealBindings> = None;

#[macro_export]
macro_rules! implement_unreal_module {
    ($module: ty) => {
        static mut MODULE: Option<$module> = None;
        #[no_mangle]
        pub unsafe extern "C" fn register_unreal_bindings(bindings: $crate::ffi::UnrealBindings) {
            $crate::module::BINDINGS = Some(bindings);
            $crate::log::init();
            MODULE = Some(<$module as $crate::module::UnrealModule>::initialize());
        }

        #[no_mangle]
        pub extern "C" fn tick(dt: f32) -> $crate::ffi::ResultCode {
            let r = std::panic::catch_unwind(|| unsafe {
                UnrealModule::tick(MODULE.as_mut().unwrap(), dt);
            });
            match r {
                Ok(_) => $crate::ffi::ResultCode::Success,
                Err(_) => $crate::ffi::ResultCode::Panic,
            }
        }

        #[no_mangle]
        pub extern "C" fn begin_play() -> $crate::ffi::ResultCode {
            let r = std::panic::catch_unwind(|| unsafe {
                UnrealModule::begin_play(MODULE.as_mut().unwrap());
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
