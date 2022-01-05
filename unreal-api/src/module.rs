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
        pub extern "C" fn tick(dt: f32) {
            unsafe {
                UnrealModule::tick(MODULE.as_mut().unwrap(), dt);
            }

        }

        #[no_mangle]
        pub extern "C" fn begin_play() {
            unsafe {
                UnrealModule::begin_play(MODULE.as_mut().unwrap());
            }
        }
    };
}

pub fn bindings() -> &'static UnrealBindings {
    unsafe { BINDINGS.as_ref().unwrap() }
}
