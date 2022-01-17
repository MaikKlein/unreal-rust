pub use unreal_ffi as ffi;
pub mod log;
pub mod core;
pub mod module;

pub use bevy_ecs as ecs;
pub use glam as math;

pub fn iterate_actors(bindings: &ffi::UnrealBindings) -> Vec<*mut ffi::AActorOpaque> {
    unsafe {
        let mut v: Vec<*mut ffi::AActorOpaque> = Vec::with_capacity(200);

        let mut len = 200;
        (bindings.iterate_actors)(v.as_mut_ptr(), &mut len);
        v.set_len(len as usize);
        v
    }
}
