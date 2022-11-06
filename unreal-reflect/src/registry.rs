use std::ffi::c_void;

use bevy_ecs::{entity::Entity, prelude::World};
use glam::{Quat, Vec3};
use serde::de::Visitor;
use unreal_ffi as ffi;

#[derive(Copy, Clone, Debug)]
pub struct UClass {
    pub ptr: *mut ffi::UObjectOpague,
}
unsafe impl Send for UClass {}
unsafe impl Sync for UClass {}

impl<'de> serde::Deserialize<'de> for UClass {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let ptr = deserializer.deserialize_str(StrVisitor)?;
        Ok(Self { ptr })
    }
}
struct StrVisitor;

impl<'de> Visitor<'de> for StrVisitor {
    type Value = *mut c_void;

    fn expecting(&self, _formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        todo!()
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let addr = str::parse::<usize>(v).unwrap();
        Ok(addr as *mut c_void)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct USound {
    pub ptr: *mut ffi::UObjectOpague,
}
unsafe impl Send for USound {}
unsafe impl Sync for USound {}

// TODO: Merge usound/uclass impl
impl<'de> serde::Deserialize<'de> for USound {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let ptr = deserializer.deserialize_str(StrVisitor)?;
        Ok(Self { ptr })
    }
}

pub enum ReflectValue {
    Float(f32),
    Vector3(Vec3),
    Bool(bool),
    Quat(Quat),
    UClass(UClass),
    USound(USound),
    Composite,
}

pub enum ReflectType {
    Float,
    Vector3,
    Bool,
    Quat,
    UClass,
    USound,
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
impl ReflectDyn for USound {
    fn name(&self) -> &'static str {
        "USound"
    }

    fn get_value(&self) -> ReflectValue {
        ReflectValue::USound(*self)
    }
}

impl ReflectStatic for USound {
    const TYPE: ReflectType = ReflectType::USound;
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
