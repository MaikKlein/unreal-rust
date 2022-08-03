use bevy_ecs::prelude::*;
use ffi::{ActorComponentPtr, ActorComponentType, EventType, Quaternion};
use std::{collections::HashMap, ffi::c_void, os::raw::c_char};

use crate::{
    ffi::{self, AActorOpaque},
    input::Input,
    math::{Quat, Vec3},
    module::{bindings, Module, UserModule},
    physics::PhysicsComponent,
    plugin::Plugin,
    register_components,
};

#[derive(Debug)]
pub enum UnrealEvent {
    ActorAdded(*mut AActorOpaque),
}

unsafe impl Send for UnrealEvent {}
unsafe impl Sync for UnrealEvent {}

pub struct UnrealCore {
    module: Module,
    unreal_events: Vec<UnrealEvent>,
}

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, module: &mut Module) {
        register_components! {
            TransformComponent,
            ActorComponent,
            PlayerInputComponent,
            ParentComponent,
            PhysicsComponent,
            => module
        };

        module
            .insert_resource(Frame::default())
            .insert_resource(Time::default())
            .insert_resource(Input::default())
            .insert_resource(ActorRegistration::default())
            .add_stage(CoreStage::PreUpdate)
            .add_stage_after(CoreStage::PreUpdate, CoreStage::Update)
            .add_stage_after(CoreStage::Update, CoreStage::PostUpdate)
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::new()
                    .with_system(update_input)
                    .with_system(download_transform_from_unreal)
                    .with_system(download_physics_from_unreal),
            )
            .add_system_set_to_stage(
                CoreStage::PostUpdate,
                SystemSet::new()
                    .with_system(upload_transform_to_unreal)
                    .with_system(upload_physics_to_unreal)
                    .with_system(process_unreal_events),
            );
    }
}

impl UnrealCore {
    pub fn new(user_module: &dyn UserModule) -> Self {
        let mut module = Module::new();
        module.add_plugin(CorePlugin);
        user_module.initialize(&mut module);
        Self {
            module,
            unreal_events: Vec::new(),
        }
    }

    pub fn begin_play(&mut self, user_module: &dyn UserModule) {
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
        *self = Self::new(user_module);

        self.module.startup.run_once(&mut self.module.world);
    }
    pub fn tick(&mut self, dt: f32) {
        if let Some(mut frame) = self.module.world.get_resource_mut::<Frame>() {
            frame.dt = dt;
        }
        if let Some(mut time) = self.module.world.get_resource_mut::<Time>() {
            time.time += dt as f64;
        }
        self.module.schedule.run_once(&mut self.module.world);
        self.module.world.clear_trackers();
    }
}

pub unsafe extern "C" fn retrieve_uuids(ptr: *mut ffi::Uuid, len: *mut usize) {
    if let Some(global) = crate::module::MODULE.as_mut() {
        if ptr.is_null() {
            *len = global.core.module.reflection_registry.uuid_set.len();
        } else {
            let slice = std::ptr::slice_from_raw_parts_mut(ptr, *len);
            for (idx, uuid) in global
                .core
                .module
                .reflection_registry
                .uuid_set
                .iter()
                .take(*len)
                .enumerate()
            {
                (*slice)[idx] = to_ffi_uuid(*uuid);
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
extern "C" fn get_field_float_value(
    uuid: ffi::Uuid,
    entity: ffi::Entity,
    idx: u32,
    out: *mut f32,
) -> u32 {
    let result = std::panic::catch_unwind(|| {
        if let Some(ReflectValue::Float(f)) = get_field_value(uuid, entity, idx) {
            unsafe {
                *out = f;
            }
            1
        } else {
            0
        }
    });
    result.unwrap_or(0)
}
extern "C" fn get_field_quat_value(
    uuid: ffi::Uuid,
    entity: ffi::Entity,
    idx: u32,
    out: *mut Quaternion,
) -> u32 {
    let result = std::panic::catch_unwind(|| {
        if let Some(ReflectValue::Quat(q)) = get_field_value(uuid, entity, idx) {
            unsafe {
                *out = q.into();
            }
            1
        } else {
            0
        }
    });
    result.unwrap_or(0)
}

extern "C" fn get_field_vector3_value(
    uuid: ffi::Uuid,
    entity: ffi::Entity,
    idx: u32,
    out: *mut ffi::Vector3,
) -> u32 {
    let result = std::panic::catch_unwind(|| {
        if let Some(ReflectValue::Vector3(v)) = get_field_value(uuid, entity, idx) {
            unsafe {
                *out = v.into();
            }
            1
        } else {
            0
        }
    });
    result.unwrap_or(0)
}
extern "C" fn get_field_bool_value(
    uuid: ffi::Uuid,
    entity: ffi::Entity,
    idx: u32,
    out: *mut u32,
) -> u32 {
    let result = std::panic::catch_unwind(|| {
        if let Some(ReflectValue::Bool(b)) = get_field_value(uuid, entity, idx) {
            unsafe {
                *out = b as u32;
            }
            1
        } else {
            0
        }
    });
    result.unwrap_or(0)
}

fn get_field_value(uuid: ffi::Uuid, entity: ffi::Entity, idx: u32) -> Option<ReflectValue> {
    let uuid = from_ffi_uuid(uuid);
    unsafe {
        let global = crate::module::MODULE.as_mut()?;
        let reflect = global.core.module.reflection_registry.reflect.get(&uuid)?;

        let entity = Entity::from_bits(entity.id);
        reflect.get_field_value(&global.core.module.world, entity, idx)
    }
}

unsafe extern "C" fn number_of_fields(uuid: ffi::Uuid, out: *mut u32) -> u32 {
    fn get_number_fields(uuid: ffi::Uuid) -> Option<u32> {
        let global = unsafe { crate::module::MODULE.as_mut() }?;
        let uuid = from_ffi_uuid(uuid);
        let reflect = global.core.module.reflection_registry.reflect.get(&uuid)?;
        Some(reflect.number_of_fields() as u32)
    }
    let result = std::panic::catch_unwind(|| {
        if let Some(count) = get_number_fields(uuid) {
            *out = count;
            1
        } else {
            0
        }
    });
    result.unwrap_or(0)
}
unsafe extern "C" fn get_type_name(
    uuid: ffi::Uuid,
    out: *mut *const c_char,
    len: *mut usize,
) -> u32 {
    fn get_type_name(uuid: ffi::Uuid) -> Option<&'static str> {
        let global = unsafe { crate::module::MODULE.as_mut() }?;
        let uuid = from_ffi_uuid(uuid);
        let reflect = global.core.module.reflection_registry.reflect.get(&uuid)?;
        Some(reflect.name())
    }
    let result = std::panic::catch_unwind(|| {
        if let Some(name) = get_type_name(uuid) {
            *out = name.as_ptr() as *const c_char;
            *len = name.len();
            1
        } else {
            0
        }
    });
    result.unwrap_or(0)
}
unsafe extern "C" fn has_component(entity: ffi::Entity, uuid: ffi::Uuid) -> u32 {
    fn has_component(entity: ffi::Entity, uuid: ffi::Uuid) -> Option<u32> {
        let global = unsafe { crate::module::MODULE.as_ref() }?;
        let uuid = from_ffi_uuid(uuid);
        let reflect = global.core.module.reflection_registry.reflect.get(&uuid)?;
        let entity = Entity::from_bits(entity.id);
        Some(reflect.has_component(&global.core.module.world, entity) as u32)
    }
    let result = std::panic::catch_unwind(|| has_component(entity, uuid).unwrap_or(0));
    result.unwrap_or(0)
}

unsafe extern "C" fn get_field_name(
    uuid: ffi::Uuid,
    idx: u32,
    out: *mut *const c_char,
    len: *mut usize,
) -> u32 {
    fn get_field_name(uuid: ffi::Uuid, idx: u32) -> Option<&'static str> {
        let global = unsafe { crate::module::MODULE.as_mut() }?;
        let uuid = from_ffi_uuid(uuid);
        let reflect = global.core.module.reflection_registry.reflect.get(&uuid)?;
        reflect.get_field_name(idx)
    }
    let result = std::panic::catch_unwind(|| {
        if let Some(name) = get_field_name(uuid, idx) {
            *out = name.as_ptr() as *const c_char;
            *len = name.len();
            1
        } else {
            0
        }
    });
    result.unwrap_or(0)
}
unsafe extern "C" fn get_field_type(
    uuid: ffi::Uuid,
    idx: u32,
    out: *mut ffi::ReflectionType,
) -> u32 {
    fn get_field_type(uuid: ffi::Uuid, idx: u32) -> Option<ffi::ReflectionType> {
        let global = unsafe { crate::module::MODULE.as_mut() }?;
        let uuid = from_ffi_uuid(uuid);
        let reflect = global.core.module.reflection_registry.reflect.get(&uuid)?;
        let ty = reflect.get_field_type(idx)?;
        Some(match ty {
            ReflectType::Bool => ffi::ReflectionType::Bool,
            ReflectType::Float => ffi::ReflectionType::Float,
            ReflectType::Vector3 => ffi::ReflectionType::Vector3,
            ReflectType::Quat => ffi::ReflectionType::Quaternion,
            ReflectType::Composite => ffi::ReflectionType::Composite,
        })
    }
    let result = std::panic::catch_unwind(|| {
        if let Some(ty) = get_field_type(uuid, idx) {
            *out = ty;
            1
        } else {
            0
        }
    });
    result.unwrap_or(0)
}

pub fn from_ffi_uuid(uuid: ffi::Uuid) -> Uuid {
    unsafe {
        let arr: [u32; 4] = [uuid.a, uuid.b, uuid.c, uuid.d];
        Uuid::from_bytes(std::mem::transmute(arr))
    }
}
pub fn to_ffi_uuid(uuid: Uuid) -> ffi::Uuid {
    unsafe {
        let [a, b, c, d]: [u32; 4] = std::mem::transmute(*uuid.as_bytes());
        ffi::Uuid { a, b, c, d }
    }
}

pub fn create_reflection_fns() -> ffi::ReflectionFns {
    ffi::ReflectionFns {
        has_component,
        get_field_bool_value,
        get_field_float_value,
        get_field_quat_value,
        get_field_vector3_value,
        number_of_fields,
        get_field_name,
        get_field_type,
        get_type_name,
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
    registry.register::<ParentComponent>();
    registry.register::<PhysicsComponent>();
}

use unreal_api::{module::ReflectionRegistry, Component};
use unreal_reflect::{
    registry::{ReflectType, ReflectValue},
    Uuid,
};
#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub struct StartupStage;

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

#[derive(Default, Debug, Copy, Clone)]
pub struct Time {
    pub time: f64,
}

#[derive(Default, Debug, Component)]
#[uuid = "5ad05c2b-7cbc-4081-8819-1997b3e13331"]
pub struct ActorComponent {
    #[reflect(skip)]
    pub actor: ActorPtr,
}

impl ActorComponent {
    pub fn set_owner(&mut self, new_owner: Option<&Self>) {
        unsafe {
            let ptr = new_owner
                .map(|comp| comp.actor.0 as *const AActorOpaque)
                .unwrap_or(std::ptr::null());
            (bindings().set_owner)(self.actor.0, ptr);
        }
    }
}

#[derive(Default, Debug, Component, Clone)]
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

#[derive(Debug, Component)]
#[uuid = "f1e22f5b-2bfe-4ce5-938b-7c093def708e"]
pub struct ParentComponent {
    #[reflect(skip)]
    pub parent: Entity,
}

impl Default for ParentComponent {
    fn default() -> Self {
        todo!()
    }
}

#[derive(Default, Debug, Component)]
#[uuid = "35256309-43b4-4459-9884-eb6e9137faf5"]
pub struct PlayerInputComponent {
    pub direction: Vec3,
}

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

fn download_physics_from_unreal(mut query: Query<&mut PhysicsComponent>) {
    for mut physics in query.iter_mut() {
        physics.download_state();
    }
}
fn upload_physics_to_unreal(mut query: Query<&mut PhysicsComponent>) {
    for mut physics in query.iter_mut() {
        physics.download_state();
    }
}
fn download_transform_from_unreal(mut query: Query<(&ActorComponent, &mut TransformComponent)>) {
    for (actor, mut transform) in query.iter_mut() {
        let mut position = ffi::Vector3::default();
        let mut rotation = ffi::Quaternion::default();
        let mut scale = ffi::Vector3::default();

        (bindings().get_spatial_data)(actor.actor.0, &mut position, &mut rotation, &mut scale);

        transform.position = position.into();
        transform.rotation = rotation.into();
        transform.scale = scale.into();
        assert!(!transform.is_nan());
    }
}

fn upload_transform_to_unreal(query: Query<(&ActorComponent, &TransformComponent)>) {
    for (actor, transform) in query.iter() {
        let is_moveable = unsafe { (bindings().is_moveable)(actor.actor.0) } > 0;
        if !is_moveable {
            continue;
        }
        assert!(!transform.is_nan());
        (bindings().set_spatial_data)(
            actor.actor.0,
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
                match event {
                    UnrealEvent::ActorAdded(actor) => {
                        let entity = commands
                            .spawn()
                            .insert_bundle((
                                ActorComponent {
                                    actor: ActorPtr(actor),
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

                        (bindings().set_entity_for_actor)(
                            actor,
                            ffi::Entity {
                                id: entity.to_bits(),
                            },
                        );
                    }
                }
            }
        }
    }
}
