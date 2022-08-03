use glam::Vec3;
use unreal_ffi as ffi;

use crate::{core::to_ffi_uuid, module::bindings};

pub trait GetEditorComponentValue: Sized {
    unsafe fn get(
        actor: *const ffi::AActorOpaque,
        uuid: unreal_reflect::Uuid,
        field: &'static str,
    ) -> Option<Self>;
}

impl GetEditorComponentValue for Vec3 {
    unsafe fn get(
        actor: *const ffi::AActorOpaque,
        uuid: unreal_reflect::Uuid,
        field: &'static str,
    ) -> Option<Self> {
        let mut data = ffi::Vector3::default();
        let code = (bindings().editor_component_fns.get_editor_component_vector)(
            actor,
            to_ffi_uuid(uuid),
            ffi::Utf8Str::from(field),
            &mut data,
        );
        if code == 1 {
            Some(data.into())
        } else {
            None
        }
    }
}
