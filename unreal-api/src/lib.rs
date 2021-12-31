//#![allow(non_upper_case_globals)]
//#![allow(non_camel_case_types)]
//#![allow(non_snake_case)]

//include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub mod ffi;
pub mod module;

pub use bevy_ecs as ecs;
pub use glam as math;

pub fn log(bindings: &ffi::UnrealBindings, text: String) {
    use std::ffi::CString;
    unsafe {
        let s = CString::new(text).unwrap();
        (bindings.log)(s.as_ptr())
    }
}
