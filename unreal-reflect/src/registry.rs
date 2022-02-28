use bevy_ecs::system::EntityCommands;
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
