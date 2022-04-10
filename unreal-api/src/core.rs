use bevy_ecs::prelude::*;
use ffi::{ActorComponentPtr, ActorComponentType, EventType};
use std::{collections::HashMap, ffi::c_void};

use crate::{
    ffi::{self, AActorOpaque},
    input::Input,
    math::{Quat, Vec3},
    module::{bindings, UserModule},
};

#[derive(Debug)]
pub enum UnrealEvent {
    ActorAdded(*mut AActorOpaque),
}

unsafe impl Send for UnrealEvent {}
unsafe impl Sync for UnrealEvent {}

pub struct UnrealCore {
    world: World,
    schedule: Schedule,
    reflection_registry: ReflectionRegistry,
    unreal_events: Vec<UnrealEvent>,
}

impl UnrealCore {
    pub fn new(module: &dyn UserModule) -> Self {
        let mut reflection_registry = ReflectionRegistry::default();
        register_core_components(&mut reflection_registry);
        module.register(&mut reflection_registry);

        let mut world = World::new();

        world.insert_resource(Frame::default());
        world.insert_resource(Input::default());
        world.insert_resource(ActorRegistration::default());

        let mut schedule = Schedule::default();
        schedule
            .add_stage(CoreStage::PreUpdate, SystemStage::single_threaded())
            .add_stage(CoreStage::Update, SystemStage::single_threaded())
            .add_stage(CoreStage::PostUpdate, SystemStage::single_threaded());
        schedule.add_system_to_stage(CoreStage::PreUpdate, update_input.system());
        schedule.add_system_to_stage(
            CoreStage::PreUpdate,
            download_transform_from_unreal.system(),
        );
        schedule.add_system_to_stage(CoreStage::PostUpdate, upload_transform_to_unreal.system());
        schedule.add_system_to_stage(CoreStage::PostUpdate, process_unreal_events.system());
        Self {
            world,
            schedule,
            reflection_registry,
            unreal_events: Vec::new(),
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

        let mut startup = Schedule::default();
        startup.add_stage(CoreStage::Startup, SystemStage::single_threaded());

        module.systems(&mut startup, &mut self.schedule);

        startup.run_once(&mut self.world);

        log::info!("BeginPlay Rust");
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
        if ptr.is_null() {
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

pub unsafe extern "C" fn unreal_event(ty: *const EventType, data: *const c_void) {
    if let Some(global) = crate::module::MODULE.as_mut() {
        match *ty {
            EventType::ActorSpawned => {
                let actor_spawned_event = data as *const ffi::ActorSpawnedEvent;
                global
                    .core
                    .unreal_events
                    .push(UnrealEvent::ActorAdded((*actor_spawned_event).actor));
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
    registry.register::<TransformComponent>();
    registry.register::<ActorComponent>();
    registry.register::<PlayerInputComponent>();
    registry.register::<MovementComponent>();
    registry.register::<CameraComponent>();
    registry.register::<ParentComponent>();
    registry.register::<PhysicsComponent>();
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
#[derive(Default, Debug, TypeUuid)]
#[uuid = "ffc10b5c-635c-43ce-8288-e3c6f6d67e36"]
pub struct PhysicsComponent {
    pub ptr: UnrealPtr<Primitive>,
    pub is_simulating: bool,
    pub velocity: Vec3,
}

impl PhysicsComponent {
    pub fn new(ptr: UnrealPtr<Primitive>) -> Self {
        let mut p = Self {
            ptr,
            ..Default::default()
        };
        p.download_state();
        p
    }
    pub fn download_state(&mut self) {
        unsafe {
            self.is_simulating = (bindings().physics_bindings.is_simulating)(self.ptr.ptr) == 1;
            self.velocity = (bindings().physics_bindings.get_velocity)(self.ptr.ptr).into();
        }
    }

    pub fn upload_state(&mut self) {
        log::info!("upload");
        unsafe {
            (bindings().physics_bindings.set_velocity)(self.ptr.ptr, self.velocity.into());
        }
    }

    pub fn add_impulse(&mut self, impulse: Vec3) {
        unsafe {
            (bindings().physics_bindings.add_impulse)(self.ptr.ptr, impulse.into());
        }
    }

    pub fn add_force(&mut self, force: Vec3) {
        unsafe {
            (bindings().physics_bindings.add_force)(self.ptr.ptr, force.into());
        }
    }
}

impl_component!(PhysicsComponent);

#[derive(Default, Debug, TypeUuid, Clone)]
#[uuid = "b8738d9e-ab21-47db-8587-4019b38e35a6"]
pub struct TransformComponent {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}
impl TransformComponent {
    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }
    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::X
    }
    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Z
    }
    pub fn is_nan(&self) -> bool {
        self.position.is_nan() || self.rotation.is_nan() || self.scale.is_nan()
    }
}

impl_component!(TransformComponent);
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
    pub is_falling: bool,
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

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct ActorPtr(pub *mut AActorOpaque);
unsafe impl Send for ActorPtr {}
unsafe impl Sync for ActorPtr {}
impl Default for ActorPtr {
    fn default() -> Self {
        Self(std::ptr::null_mut())
    }
}
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnrealPtr<T> {
    pub ptr: *mut c_void,
    _m: std::marker::PhantomData<T>,
}
impl<T> UnrealPtr<T> {
    pub fn from_raw(ptr: *mut c_void) -> Self {
        Self {
            ptr,
            ..Default::default()
        }
    }
}
unsafe impl<T> Send for UnrealPtr<T> {}
unsafe impl<T> Sync for UnrealPtr<T> {}
impl<T> Default for UnrealPtr<T> {
    fn default() -> Self {
        Self {
            ptr: std::ptr::null_mut(),
            _m: Default::default(),
        }
    }
}
impl<T> Clone for UnrealPtr<T> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            _m: self._m,
        }
    }
}

impl<T> Copy for UnrealPtr<T> {}

#[derive(Debug)]
pub enum Capsule {}
#[derive(Debug)]
pub enum Primitive {}

fn download_transform_from_unreal(mut query: Query<(&ActorComponent, &mut TransformComponent)>) {
    for (actor, mut transform) in query.iter_mut() {
        let mut position = ffi::Vector3::default();
        let mut rotation = ffi::Quaternion::default();
        let mut scale = ffi::Vector3::default();

        (bindings().get_spatial_data)(actor.ptr.0, &mut position, &mut rotation, &mut scale);

        transform.position = position.into();
        transform.rotation = rotation.into();
        transform.scale = scale.into();
        assert!(!transform.is_nan());
    }
}

fn upload_transform_to_unreal(query: Query<(&ActorComponent, &TransformComponent)>) {
    for (actor, transform) in query.iter() {
        let is_moveable = unsafe { (bindings().is_moveable)(actor.ptr.0) } > 0;
        if !is_moveable {
            continue;
        }
        assert!(!transform.is_nan());
        (bindings().set_spatial_data)(
            actor.ptr.0,
            transform.position.into(),
            transform.rotation.into(),
            transform.scale.into(),
        );
    }
}

fn update_input(mut input: ResMut<Input>) {
    input.update();
}

fn process_unreal_events(mut actor_register: ResMut<ActorRegistration>, mut commands: Commands) {
    unsafe {
        // TODO: This is UB
        if let Some(global) = crate::module::MODULE.as_mut() {
            for event in global.core.unreal_events.drain(..) {
                log::info!("{:?}", event);
                match event {
                    UnrealEvent::ActorAdded(actor) => {
                        let entity = commands
                            .spawn()
                            .insert_bundle((
                                ActorComponent {
                                    ptr: ActorPtr(actor),
                                },
                                TransformComponent::default(),
                            ))
                            .id();

                        let mut root_component = ActorComponentPtr::default();
                        (bindings().get_root_component)(actor, &mut root_component);
                        if root_component.ty == ActorComponentType::Primitive
                            && !root_component.ptr.is_null()
                        {
                            let physics_component =
                                PhysicsComponent::new(UnrealPtr::from_raw(root_component.ptr));
                            commands.entity(entity).insert(physics_component);
                        }

                        actor_register
                            .actor_to_entity
                            .insert(ActorPtr(actor), entity);
                    }
                }
            }
        }
    }
}
