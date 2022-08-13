use bevy_ecs::{entity::Entity, prelude::World};
use glam::{Quat, Vec3};
use unreal_ffi as ffi;


#[derive(Copy, Clone, Debug)]
pub struct UClass {
    pub ptr: *mut ffi::UObjectOpague,
}
unsafe impl Send for UClass{}
unsafe impl Sync for UClass{}

pub enum ReflectValue {
    Float(f32),
    Vector3(Vec3),
    Bool(bool),
    Quat(Quat),
    UClass(UClass),
    Composite,
}

pub enum ReflectType {
    Float,
    Vector3,
    Bool,
    Quat,
    UClass,
    Composite,
}

pub trait ReflectDyn {
    fn name(&self) -> &'static str;
    fn number_of_fields(&self) -> u32 {
        0
    }
    fn get_field_name(&self, _idx: u32) -> Option<&'static str> {
        None
    }
    fn get_field_type(&self, _idx: u32) -> Option<ReflectType> {
        None
    }
    fn has_component(&self, _world: &World, _entity: Entity) -> bool {
        false
    }
    fn get_field_value(&self, _world: &World, _entity: Entity, _idx: u32) -> Option<ReflectValue> {
        None
    }
    fn get_value(&self) -> ReflectValue;
}

impl ReflectDyn for UClass {
    fn name(&self) -> &'static str {
        "UClass"
    }

    fn get_value(&self) -> ReflectValue {
        ReflectValue::UClass(*self)
    }
}
impl ReflectStatic for UClass {
    const TYPE: ReflectType = ReflectType::UClass;
}

impl ReflectDyn for Vec3 {
    fn name(&self) -> &'static str {
        "Vec3"
    }

    fn get_value(&self) -> ReflectValue {
        ReflectValue::Vector3(*self)
    }
}

impl ReflectStatic for Vec3 {
    const TYPE: ReflectType = ReflectType::Vector3;
}

impl ReflectDyn for Quat {
    fn name(&self) -> &'static str {
        "Quat"
    }

    fn get_value(&self) -> ReflectValue {
        ReflectValue::Quat(*self)
    }
}
impl ReflectStatic for Quat {
    const TYPE: ReflectType = ReflectType::Quat;
}

impl ReflectDyn for f32 {
    fn name(&self) -> &'static str {
        "f32"
    }

    fn get_value(&self) -> ReflectValue {
        ReflectValue::Float(*self)
    }
}

impl ReflectStatic for f32 {
    const TYPE: ReflectType = ReflectType::Float;
}
impl ReflectDyn for bool {
    fn name(&self) -> &'static str {
        "bool"
    }

    fn get_value(&self) -> ReflectValue {
        ReflectValue::Bool(*self)
    }
}

impl ReflectStatic for bool {
    const TYPE: ReflectType = ReflectType::Bool;
}

pub trait ReflectStatic {
    const TYPE: ReflectType;
}
