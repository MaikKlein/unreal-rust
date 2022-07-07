use bevy_ecs::schedule::Schedule;
use unreal_reflect::registry::ReflectionRegistry;

use crate::{core::UnrealCore, ffi::UnrealBindings};

pub static mut MODULE: Option<Global> = None;
pub struct Global {
    pub core: UnrealCore,
    pub module: Box<dyn UserModule>,
}

pub trait InitUserModule {
    fn initialize() -> Self;
}
pub trait UserModule {
    fn register(&self, registry: &mut ReflectionRegistry);
    fn systems(&self, startup: &mut Schedule, update: &mut Schedule);
}
pub static mut BINDINGS: Option<UnrealBindings> = None;

#[macro_export]
macro_rules! implement_unreal_module {
    ($module: ty) => {
        #[no_mangle]
        pub unsafe extern "C" fn register_unreal_bindings(
            bindings: $crate::ffi::UnrealBindings,
        ) -> $crate::ffi::RustBindings {
            $crate::module::BINDINGS = Some(bindings);
            let _ = $crate::log::init();
            let module = Box::new(<$module as $crate::module::InitUserModule>::initialize());
            let core = $crate::core::UnrealCore::new(module.as_ref());

            $crate::module::MODULE = Some($crate::module::Global { core, module });
            $crate::ffi::RustBindings {
                retrieve_uuids: $crate::core::retrieve_uuids,
                tick: $crate::core::tick,
                begin_play: $crate::core::begin_play,
                unreal_event: $crate::core::unreal_event,
                reflection_fns: $crate::core::create_reflection_fns(),
            }
        }
    };
}

pub fn bindings() -> &'static UnrealBindings {
    unsafe { BINDINGS.as_ref().unwrap() }
}
