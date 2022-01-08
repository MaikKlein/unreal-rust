use bevy_ecs::prelude::*;
use std::{collections::HashMap, ffi::CStr};

use crate::{
    ffi::{self, AActorOpaque, ActionState},
    iterate_actors,
    math::{Quat, Vec3},
    module::{bindings, UnrealModule},
};
pub struct UnrealCore<Module> {
    world: World,
    schedule: Schedule,
    startup: Schedule,
    module: Module,
    reflection_registry: ReflectionRegistry,
}

impl<Module: UnrealModule> UnrealCore<Module> {
    pub fn new() -> Self {
        log::info!("Initialize Rust");
        let mut startup = Schedule::default();
        startup.add_stage(CoreStage::Startup, SystemStage::single_threaded());

        let mut schedule = Schedule::default();
        schedule
            .add_stage(CoreStage::PreUpdate, SystemStage::single_threaded())
            .add_stage(CoreStage::Update, SystemStage::single_threaded())
            .add_stage(CoreStage::PostUpdate, SystemStage::single_threaded());

        schedule.add_system_to_stage(CoreStage::PreUpdate, download_spatial_from_unreal.system());
        schedule.add_system_to_stage(CoreStage::PostUpdate, upload_spatial_to_unreal.system());

        let mut reflection_registry = ReflectionRegistry::default();
        Module::register(&mut reflection_registry);
        Module::systems(&mut startup, &mut schedule);
        Self {
            world: World::new(),
            schedule,
            startup,
            module: Module::initialize(),
            reflection_registry,
        }
    }

    pub fn begin_play(&mut self) {
        std::panic::set_hook(Box::new(|panic_info| {
            if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                let location = panic_info.location().map_or("".to_string(), |loc| {
                    format!("{}, at line {}", loc.file(), loc.line())
                });
                log::error!("Panic: {} => {}", location, s);
            } else {
                log::error!("panic occurred");
            }
        }));
        *self = Self::new();
        log::info!("BeginPlay Rust");
        self.world.insert_resource(Frame::default());
        self.world.insert_resource(ActorRegistration::default());
        let mut startup = Schedule::default();
        startup.add_stage(CoreStage::Startup, SystemStage::single_threaded());
        startup.add_system_to_stage(CoreStage::Startup, register_actors.system());
        startup.run_once(&mut self.world);
        self.startup.run_once(&mut self.world);
    }
    pub fn tick(&mut self, dt: f32) {
        if let Some(mut frame) = self.world.get_resource_mut::<Frame>() {
            frame.dt = dt;
        }
        self.schedule.run_once(&mut self.world);
    }
}
impl<M> UnrealCore<M> {
    pub unsafe fn retrieve_uuids(&self, ptr: *mut ffi::Uuid, len: *mut usize) {
        if ptr == std::ptr::null_mut() {
            *len = self.reflection_registry.uuid_set.len();
        } else {
            let slice = std::ptr::slice_from_raw_parts_mut(ptr, *len);
            for (idx, uuid) in self
                .reflection_registry
                .uuid_set
                .iter()
                .take(*len)
                .enumerate()
            {
                (*slice)[idx] = ffi::Uuid {
                    bytes: *uuid.as_bytes(),
                };
            }
        }
    }
}

pub fn register_core_components(registry: &mut ReflectionRegistry) {
    registry.register::<SpatialComponent>();
    registry.register::<ActorComponent>();
    registry.register::<PlayerInputComponent>();
    registry.register::<MovementComponent>();
}
use unreal_reflect::{impl_component, registry::ReflectionRegistry, TypeUuid};
#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum CoreStage {
    Startup,
    PreUpdate,
    Update,
    PostUpdate,
}
#[derive(Default, Debug, Copy, Clone)]
pub struct Frame {
    pub dt: f32,
}

#[derive(Default, Debug, TypeUuid)]
#[uuid = "b8738d9e-ab21-47db-8587-4019b38e35a6"]
pub struct ActorComponent {
    ptr: ActorPtr,
}
impl_component!(ActorComponent);

#[derive(Default, Debug, TypeUuid)]
#[uuid = "b8738d9e-ab21-47db-8587-4019b38e35a6"]
pub struct SpatialComponent {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}
impl_component!(SpatialComponent);

#[derive(Default, Debug, TypeUuid)]
#[uuid = "fc8bd668-fc0a-4ab7-8b3d-f0f22bb539e2"]
pub struct MovementComponent {
    pub velocity: Vec3,
}
impl_component!(MovementComponent);

#[derive(Default, Debug, TypeUuid)]
#[uuid = "35256309-43b4-4459-9884-eb6e9137faf5"]
pub struct PlayerInputComponent {
    pub direction: Vec3,
}
impl_component!(PlayerInputComponent);

// TODO: Implement unregister.
#[derive(Default)]
pub struct ActorRegistration {
    pub actor_to_entity: HashMap<ActorPtr, Entity>,
}

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ActorPtr(pub *mut AActorOpaque);
unsafe impl Send for ActorPtr {}
unsafe impl Sync for ActorPtr {}
impl Default for ActorPtr {
    fn default() -> Self {
        Self(std::ptr::null_mut())
    }
}

fn download_spatial_from_unreal(mut query: Query<(&ActorComponent, &mut SpatialComponent)>) {
    for (actor, mut spatial) in query.iter_mut() {
        let mut position = ffi::Vector3::default();
        let mut rotation = ffi::Quaternion::default();
        let mut scale = ffi::Vector3::default();

        (bindings().get_spatial_data)(actor.ptr.0, &mut position, &mut rotation, &mut scale);

        spatial.position = position.into();
        spatial.rotation = rotation.into();
        spatial.scale = scale.into();
    }
}
fn upload_spatial_to_unreal(query: Query<(&ActorComponent, &SpatialComponent)>) {
    for (actor, spatial) in query.iter() {
        (bindings().set_spatial_data)(
            actor.ptr.0,
            spatial.position.into(),
            spatial.rotation.into(),
            spatial.scale.into(),
        );
    }
}

fn register_actors(mut actor_register: ResMut<ActorRegistration>, mut commands: Commands) {
    for actor in iterate_actors(bindings()) {
        log::info!("actor");
        let entity = commands
            .spawn()
            .insert_bundle((
                ActorComponent {
                    ptr: ActorPtr(actor),
                },
                SpatialComponent::default(),
                MovementComponent::default(),
                PlayerInputComponent::default(),
            ))
            .id();

        actor_register
            .actor_to_entity
            .insert(ActorPtr(actor), entity);
    }
}
