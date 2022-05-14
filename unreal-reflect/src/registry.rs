use bevy_ecs::{entity::Entity, prelude::World, system::EntityCommands};
use glam::Vec3;
use std::collections::{HashMap, HashSet};

use crate::TypeUuid;

#[macro_export]
macro_rules! impl_component {
    ($component: ty) => {
        $crate::impl_insert_with_default!($component);
        $crate::impl_reflection!($component);
        impl $crate::bevy_ecs::component::Component for $component {
            type Storage = $crate::bevy_ecs::component::TableStorage;
        }
    };
}
#[macro_export]
macro_rules! impl_insert_with_default {
    ($component: ty) => {
        impl $crate::registry::InsertComponent for $component {
            fn insert(commands: &'_ mut $crate::bevy_ecs::system::EntityCommands<'_, '_, '_>) {
                commands.insert(<$component>::default());
            }
        }
    };
}
#[macro_export]
macro_rules! impl_reflection {
    ($component: ty) => {
        impl $crate::registry::Reflection for $component {
            fn reflection() -> $crate::registry::ReflectionData {
                $crate::registry::ReflectionData {
                    name: std::any::type_name::<$component>(),
                }
            }
        }
    };
}

pub type InsertComponentFn = Box<dyn Fn(&'_ mut EntityCommands<'_, '_, '_>)>;
#[derive(Default)]
pub struct ReflectionRegistry {
    pub uuid_to_insert_component: HashMap<uuid::Uuid, InsertComponentFn>,
    pub uuid_to_refection_data: HashMap<uuid::Uuid, ReflectionData>,
    pub uuid_set: HashSet<uuid::Uuid>,
    pub reflect: HashMap<uuid::Uuid, Box<dyn Reflect>>,
}

impl ReflectionRegistry {
    pub fn register<T>(&mut self)
    where
        T: Reflection + InsertComponent + TypeUuid + 'static,
    {
        if self.uuid_set.contains(&T::TYPE_UUID) {
            //TODO: Log not ready here
            log::error!(
                "Duplicated UUID {} for {}",
                T::TYPE_UUID,
                std::any::type_name::<T>()
            );
            return;
        }
        self.uuid_to_insert_component
            .insert(T::TYPE_UUID, Box::new(T::insert));
        self.uuid_to_refection_data
            .insert(T::TYPE_UUID, T::reflection());
        self.uuid_set.insert(T::TYPE_UUID);
    }
}

pub trait InsertComponent {
    fn insert(commands: &'_ mut EntityCommands<'_, '_, '_>);
}
#[derive(Debug)]
pub struct ReflectionData {
    pub name: &'static str,
}

pub trait Reflection {
    fn reflection() -> ReflectionData;
}
#[derive(Debug)]
pub struct ReflectionData2 {
    pub name: &'static str,
    pub number_of_fields: u32,
}

pub enum ReflectValue {
    Float(f32),
    Vector3(Vec3),
    Bool(bool),
}

pub enum ReflectType {
    Float,
    Vector3,
    Bool,
}

pub trait Reflect {
    fn name(&self) -> &'static str;
    fn number_of_fields(&self) -> u32;
    fn get_field_name(&self, idx: u32) -> Option<&'static str>;
    fn get_field_type(&self, idx: u32) -> Option<ReflectType>;
    fn get_field_value(&self, world: &World, entity: Entity, idx: u32) -> Option<ReflectValue>;
}
