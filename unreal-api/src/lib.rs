#![allow(clippy::missing_safety_doc)]
extern crate self as unreal_api;

pub use unreal_ffi as ffi;
pub mod core;
pub mod editor_component;
pub mod input;
pub mod log;
pub mod module;
pub mod physics;
pub mod plugin;
pub use unreal_api_derive::Component;

// TODO: Here for the unreal_api_derive macro. Lets restructure this
pub use unreal_reflect::*;
pub use bevy_ecs as ecs;
pub use glam as math;

pub use uuid;

pub fn iterate_actors(bindings: &ffi::UnrealBindings) -> Vec<*mut ffi::AActorOpaque> {
    unsafe {
        let mut v: Vec<*mut ffi::AActorOpaque> = Vec::with_capacity(200);

        let mut len = 200;
        (bindings.iterate_actors)(v.as_mut_ptr(), &mut len);
        v.set_len(len as usize);
        v
    }
}
