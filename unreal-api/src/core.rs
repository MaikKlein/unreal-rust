use bevy_ecs::prelude::*;
use std::collections::HashMap;

use crate::{
    ffi::{self, AActorOpaque},
    iterate_actors,
    math::{Quat, Vec3},
    module::{bindings, UserModule},
};
pub struct UnrealCore {
    world: World,
    schedule: Schedule,
    startup: Schedule,
    reflection_registry: ReflectionRegistry,
}

impl UnrealCore {
    pub fn new(module: &dyn UserModule) -> Self {
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
        register_core_components(&mut reflection_registry);
        module.register(&mut reflection_registry);
        module.systems(&mut startup, &mut schedule);
        Self {
            world: World::new(),
            schedule,
            startup,
            reflection_registry,
        }
    }

    pub fn begin_play(&mut self, module: &dyn UserModule) {
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
        *self = Self::new(module);
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
        self.world.clear_trackers();
    }
}

pub unsafe extern "C" fn retrieve_uuids(ptr: *mut ffi::Uuid, len: *mut usize) {
    if let Some(global) = crate::module::MODULE.as_mut() {
        if ptr == std::ptr::null_mut() {
            *len = global.core.reflection_registry.uuid_set.len();
        } else {
            let slice = std::ptr::slice_from_raw_parts_mut(ptr, *len);
            for (idx, uuid) in global
                .core
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

pub unsafe extern "C" fn get_velocity(actor: *const AActorOpaque, velocity: &mut ffi::Vector3) {
    if let Some(global) = crate::module::MODULE.as_mut() {
        if let Some(entity) = global
            .core
            .world
            .get_resource::<ActorRegistration>()
            .and_then(|reg| reg.actor_to_entity.get(&ActorPtr(actor as *mut _)))
            .copied()
        {
            if let Some(movement) = global
                .core
                .world
                .get_entity(entity)
                .and_then(|eref| eref.get::<MovementComponent>())
            {
                *velocity = movement.velocity.into();
            }
        }
    }
}
pub extern "C" fn tick(dt: f32) -> crate::ffi::ResultCode {
    let r = std::panic::catch_unwind(|| unsafe {
        UnrealCore::tick(&mut crate::module::MODULE.as_mut().unwrap().core, dt);
    });
    match r {
        Ok(_) => ffi::ResultCode::Success,
        Err(_) => ffi::ResultCode::Panic,
    }
}

pub extern "C" fn begin_play() -> ffi::ResultCode {
    let r = std::panic::catch_unwind(|| unsafe {
        let global = crate::module::MODULE.as_mut().unwrap();
        UnrealCore::begin_play(&mut global.core, global.module.as_ref());
    });
    match r {
        Ok(_) => ffi::ResultCode::Success,
        Err(_) => ffi::ResultCode::Panic,
    }
}
pub fn register_core_components(registry: &mut ReflectionRegistry) {
    registry.register::<SpatialComponent>();
    registry.register::<ActorComponent>();
    registry.register::<PlayerInputComponent>();
    registry.register::<MovementComponent>();
    registry.register::<CameraComponent>();
    registry.register::<ParentComponent>();
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
#[uuid = "5ad05c2b-7cbc-4081-8819-1997b3e13331"]
pub struct ActorComponent {
    pub ptr: ActorPtr,
}
impl_component!(ActorComponent);

#[derive(Default, Debug, TypeUuid, Clone)]
#[uuid = "b8738d9e-ab21-47db-8587-4019b38e35a6"]
pub struct SpatialComponent {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}
impl_component!(SpatialComponent);
#[derive(Default, Debug, TypeUuid)]
#[uuid = "8d2df877-499b-46f3-9660-bd2e1867af0d"]
pub struct CameraComponent {
    pub x: f32,
    pub y: f32,
    pub current_x: f32,
    pub current_y: f32,
}
impl_component!(CameraComponent);

#[derive(Default, Debug, TypeUuid)]
#[uuid = "fc8bd668-fc0a-4ab7-8b3d-f0f22bb539e2"]
pub struct MovementComponent {
    pub velocity: Vec3,
    pub view: Quat,
}
impl_component!(MovementComponent);

#[derive(Debug, TypeUuid)]
#[uuid = "f1e22f5b-2bfe-4ce5-938b-7c093def708e"]
pub struct ParentComponent {
    pub parent: Entity,
}
impl_component!(ParentComponent);

impl Default for ParentComponent {
    fn default() -> Self {
        todo!()
    }
}

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
