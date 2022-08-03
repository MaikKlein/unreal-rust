use std::collections::{HashMap, HashSet};

use bevy_ecs::{
    prelude::System,
    schedule::{Schedule, StageLabel, SystemSet, SystemStage},
    system::Resource,
};
use unreal_reflect::{registry::ReflectDyn, uuid, TypeUuid, World};

use crate::{
    core::{StartupStage, UnrealCore},
    editor_component::InsertEditorComponent,
    ffi::UnrealBindings,
    plugin::Plugin,
};

pub static mut MODULE: Option<Global> = None;
pub struct Global {
    pub core: UnrealCore,
    pub module: Box<dyn UserModule>,
}

pub trait InitUserModule {
    fn initialize() -> Self;
}

pub type EmptySystem = &'static dyn System<In = (), Out = ()>;
#[macro_export]
macro_rules! register_components {
    ($($ty: ty,)* => $module: expr) => {
        $(
            $module.register_component::<$ty>();
        )*
    };
}
pub trait InsertReflectionStruct {
    fn insert(registry: &mut ReflectionRegistry);
}

#[derive(Default)]
pub struct ReflectionRegistry {
    pub uuid_set: HashSet<uuid::Uuid>,
    pub reflect: HashMap<uuid::Uuid, Box<dyn ReflectDyn>>,
    pub insert_editor_component: HashMap<uuid::Uuid, Box<dyn InsertEditorComponent>>,
}

impl ReflectionRegistry {
    pub fn register<T>(&mut self)
    where
        T: InsertReflectionStruct + TypeUuid + 'static,
    {
        if self.uuid_set.contains(&T::TYPE_UUID) {
            panic!(
                "Duplicated UUID {} for {}",
                T::TYPE_UUID,
                std::any::type_name::<T>()
            );
        }
        T::insert(self);
        self.uuid_set.insert(T::TYPE_UUID);
    }
}

pub struct Module {
    pub(crate) schedule: Schedule,
    pub(crate) startup: Schedule,
    pub(crate) reflection_registry: ReflectionRegistry,
    pub(crate) world: World,
}

impl Module {
    pub fn new() -> Self {
        let mut startup = Schedule::default();
        startup.add_stage(StartupStage, SystemStage::single_threaded());

        Self {
            schedule: Schedule::default(),
            startup,
            reflection_registry: ReflectionRegistry::default(),
            world: World::new(),
        }
    }
    pub fn insert_resource(&mut self, resource: impl Resource) -> &mut Self {
        self.world.insert_resource(resource);
        self
    }

    pub fn add_stage(&mut self, label: impl StageLabel) -> &mut Self {
        self.schedule
            .add_stage(label, SystemStage::single_threaded());
        self
    }

    pub fn add_stage_after(
        &mut self,
        label: impl StageLabel,
        insert: impl StageLabel,
    ) -> &mut Self {
        self.schedule
            .add_stage_after(label, insert, SystemStage::single_threaded());
        self
    }

    pub fn add_stage_before(
        &mut self,
        label: impl StageLabel,
        insert: impl StageLabel,
    ) -> &mut Self {
        self.schedule
            .add_stage_before(label, insert, SystemStage::single_threaded());
        self
    }

    pub fn add_system_set_to_stage(&mut self, label: impl StageLabel, set: SystemSet) -> &mut Self {
        self.schedule.add_system_set_to_stage(label, set);
        self
    }

    pub fn register_component<T>(&mut self)
    where
        T: InsertReflectionStruct + TypeUuid + 'static,
    {
        self.reflection_registry.register::<T>();
    }

    pub fn add_plugin<P: Plugin>(&mut self, plugin: P) -> &mut Self {
        plugin.build(self);
        self
    }

    pub fn add_startup_system_set(&mut self, system_set: SystemSet) -> &mut Self {
        self.startup
            .add_system_set_to_stage(StartupStage, system_set);
        self
    }
}

impl Default for Module {
    fn default() -> Self {
        Self::new()
    }
}

pub trait UserModule {
    fn initialize(&self, module: &mut Module);
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
