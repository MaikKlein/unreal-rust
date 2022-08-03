use bevy_ecs::system::EntityCommands;
use glam::{Quat, Vec3};
use unreal_ffi as ffi;
use unreal_reflect::Uuid;

use crate::{core::to_ffi_uuid, module::bindings};

pub trait InsertEditorComponent {
    /// # Safety
    unsafe fn insert_component(
        &self,
        actor: *const ffi::AActorOpaque,
        uuid: Uuid,
        commands: &mut EntityCommands<'_, '_, '_>,
    );
}

pub trait GetEditorComponentValue: Sized {
    unsafe fn get(actor: *const ffi::AActorOpaque, uuid: Uuid, field: &'static str)
        -> Option<Self>;
}

impl GetEditorComponentValue for Vec3 {
    unsafe fn get(
        actor: *const ffi::AActorOpaque,
        uuid: Uuid,
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
impl GetEditorComponentValue for Quat {
    unsafe fn get(
        actor: *const ffi::AActorOpaque,
        uuid: Uuid,
        field: &'static str,
    ) -> Option<Self> {
        let mut data = ffi::Quaternion::default();
        let code = (bindings().editor_component_fns.get_editor_component_quat)(
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

impl GetEditorComponentValue for f32 {
    unsafe fn get(
        actor: *const ffi::AActorOpaque,
        uuid: Uuid,
        field: &'static str,
    ) -> Option<Self> {
        let mut data = 0.0f32;
        let code = (bindings().editor_component_fns.get_editor_component_float)(
            actor,
            to_ffi_uuid(uuid),
            ffi::Utf8Str::from(field),
            &mut data,
        );
        if code == 1 {
            Some(data)
        } else {
            None
        }
    }
}

impl GetEditorComponentValue for bool {
    unsafe fn get(
        actor: *const ffi::AActorOpaque,
        uuid: Uuid,
        field: &'static str,
    ) -> Option<Self> {
        let mut data: u32 = 0;
        let code = (bindings().editor_component_fns.get_editor_component_bool)(
            actor,
            to_ffi_uuid(uuid),
            ffi::Utf8Str::from(field),
            &mut data,
        );
        if code == 1 {
            Some(data == 1)
        } else {
            None
        }
    }
}
