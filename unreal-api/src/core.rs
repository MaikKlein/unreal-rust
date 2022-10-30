use bevy_ecs::prelude::*;
use std::ffi::c_void;

use std::panic;

use bevy_ecs::schedule::{StageLabel, SystemSet};
use bevy_ecs::system::Command;
use ffi::{EventType, Quaternion, StrRustAlloc};
use unreal_api::{module::ReflectionRegistry, Component};
use unreal_reflect::{
    registry::{ReflectType, ReflectValue},
    Entity, Uuid, World,
};

use crate::{
    api::UnrealApi,
    ffi::{self, AActorOpaque},
    input::Input,
    math::{Quat, Vec3},
    module::{bindings, Module, UserModule},
    physics::PhysicsComponent,
    plugin::Plugin,
    register_components,
};

pub struct UnrealCore {
    module: Module,
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
            .insert_resource(UnrealApi::default())
            .add_stage(CoreStage::RegisterEvent)
            .add_stage_after(CoreStage::RegisterEvent, CoreStage::PreUpdate)
            .add_stage_after(CoreStage::PreUpdate, CoreStage::Update)
            .add_stage_after(CoreStage::Update, CoreStage::PostUpdate)
            // TODO: Order matters here. Needs to be defined after the stages
            .add_event::<OnActorBeginOverlapEvent>()
            .add_event::<OnActorEndOverlapEvent>()
            .add_event::<ActorHitEvent>()
            .add_event::<ActorSpawnedEvent>()
            .add_event::<ActorDestroyEvent>()
            .add_system_set_to_stage(
                CoreStage::RegisterEvent,
                SystemSet::new()
                    .with_system(process_actor_spawned)
                    .with_system(process_actor_destroyed),
            )
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
                    .with_system(upload_physics_to_unreal),
            );
    }
}

impl UnrealCore {
    pub fn new(user_module: &dyn UserModule) -> Self {
        let mut module = Module::new();
        module.add_plugin(CorePlugin);
        user_module.initialize(&mut module);
        Self { module }
    }

    pub fn begin_play(&mut self, user_module: &dyn UserModule) {
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
        //self.module.world.clear_trackers();
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

pub struct ActorSpawnedEvent {
    pub actor: ActorPtr,
}

pub struct OnActorBeginOverlapEvent {
    pub overlapped_actor: ActorPtr,
    pub other: ActorPtr,
}

pub struct OnActorEndOverlapEvent {
    pub overlapped_actor: ActorPtr,
    pub other: ActorPtr,
}

pub struct ActorHitEvent {
    pub self_actor: ActorPtr,
    pub other: ActorPtr,
    pub normal_impulse: Vec3,
}

pub struct ActorDestroyEvent {
    pub actor: ActorPtr,
}

pub struct EntityEvent<E> {
    pub entity: Entity,
    pub event: E,
}

pub trait SendEntityEvent {
    fn send_entity_event(&self, world: &mut World, entity: Entity, json: &str);
}

pub unsafe extern "C" fn send_actor_event(
    actor: *const ffi::AActorOpaque,
    uuid: ffi::Uuid,
    json: ffi::Utf8Str,
) {
    log::info!("send event {}", from_ffi_uuid(uuid));
    let _ = std::panic::catch_unwind(|| {
        if let Some(global) = crate::module::MODULE.as_mut() {
            if let Some(send_event) = global
                .core
                .module
                .reflection_registry
                .send_entity_event
                .get(&from_ffi_uuid(uuid))
            {
                let api = global
                    .core
                    .module
                    .world
                    .get_resource::<UnrealApi>()
                    .unwrap();
                let entity = *api.actor_to_entity.get(&ActorPtr(actor as _)).unwrap();

                log::info!("send event 2");
                send_event.send_entity_event(&mut global.core.module.world, entity, json.as_str());
            }
        }
    });
}

pub unsafe extern "C" fn unreal_event(ty: *const EventType, data: *const c_void) {
    if let Some(global) = crate::module::MODULE.as_mut() {
        match *ty {
            EventType::ActorSpawned => {
                let actor_spawned_event = data as *const ffi::ActorSpawnedEvent;
                global.core.module.world.send_event(ActorSpawnedEvent {
                    actor: ActorPtr((*actor_spawned_event).actor),
                });
            }
            EventType::ActorBeginOverlap => {
                let overlap = data as *const ffi::ActorBeginOverlap;
                global
                    .core
                    .module
                    .world
                    .send_event(OnActorBeginOverlapEvent {
                        overlapped_actor: ActorPtr((*overlap).overlapped_actor),
                        other: ActorPtr((*overlap).other),
                    });
            }
            EventType::ActorEndOverlap => {
                let overlap = data as *const ffi::ActorEndOverlap;
                global.core.module.world.send_event(OnActorEndOverlapEvent {
                    overlapped_actor: ActorPtr((*overlap).overlapped_actor),
                    other: ActorPtr((*overlap).other),
                });
            }
            EventType::ActorOnHit => {
                let hit = data as *const ffi::ActorHitEvent;
                global.core.module.world.send_event(ActorHitEvent {
                    self_actor: ActorPtr((*hit).self_actor),
                    other: ActorPtr((*hit).other),
                    normal_impulse: (*hit).normal_impulse.into(),
                });
            }
            EventType::ActorDestroy => {
                let destroy = data as *const ffi::ActorDestroyEvent;
                global.core.module.world.send_event(ActorDestroyEvent {
                    actor: ActorPtr((*destroy).actor),
                });
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
    let result = panic::catch_unwind(|| {
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
    let result = panic::catch_unwind(|| {
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
    let result = panic::catch_unwind(|| {
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
    let result = panic::catch_unwind(|| {
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
    let result = panic::catch_unwind(|| {
        if let Some(count) = get_number_fields(uuid) {
            *out = count;
            1
        } else {
            0
        }
    });
    result.unwrap_or(0)
}
unsafe extern "C" fn get_type_name(uuid: ffi::Uuid, out: *mut ffi::Utf8Str) -> u32 {
    fn get_type_name(uuid: ffi::Uuid) -> Option<&'static str> {
        let global = unsafe { crate::module::MODULE.as_mut() }?;
        let uuid = from_ffi_uuid(uuid);
        let reflect = global.core.module.reflection_registry.reflect.get(&uuid)?;
        Some(reflect.name())
    }
    let result = panic::catch_unwind(|| {
        if let Some(name) = get_type_name(uuid) {
            *out = ffi::Utf8Str::from(name);
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
    let result = panic::catch_unwind(|| has_component(entity, uuid).unwrap_or(0));
    result.unwrap_or(0)
}
unsafe extern "C" fn is_event(uuid: ffi::Uuid) -> u32 {
    fn is_editor_component_inner(uuid: ffi::Uuid) -> Option<u32> {
        let global = unsafe { crate::module::MODULE.as_mut() }?;
        let uuid = from_ffi_uuid(uuid);
        Some(
            if global
                .core
                .module
                .reflection_registry
                .events
                .contains(&uuid)
            {
                1
            } else {
                0
            },
        )
    }
    let result = panic::catch_unwind(|| is_editor_component_inner(uuid).unwrap_or(0));
    result.unwrap_or(0)
}

unsafe extern "C" fn is_editor_component(uuid: ffi::Uuid) -> u32 {
    fn is_editor_component_inner(uuid: ffi::Uuid) -> Option<u32> {
        let global = unsafe { crate::module::MODULE.as_mut() }?;
        let uuid = from_ffi_uuid(uuid);
        Some(
            if global
                .core
                .module
                .reflection_registry
                .editor_components
                .contains(&uuid)
            {
                1
            } else {
                0
            },
        )
    }
    let result = panic::catch_unwind(|| is_editor_component_inner(uuid).unwrap_or(0));
    result.unwrap_or(0)
}

unsafe extern "C" fn get_field_name(uuid: ffi::Uuid, idx: u32, out: *mut ffi::Utf8Str) -> u32 {
    fn get_field_name(uuid: ffi::Uuid, idx: u32) -> Option<&'static str> {
        let global = unsafe { crate::module::MODULE.as_mut() }?;
        let uuid = from_ffi_uuid(uuid);
        let reflect = global.core.module.reflection_registry.reflect.get(&uuid)?;
        reflect.get_field_name(idx)
    }
    let result = panic::catch_unwind(|| {
        if let Some(name) = get_field_name(uuid, idx) {
            *out = ffi::Utf8Str::from(name);
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
            ReflectType::UClass => ffi::ReflectionType::UClass,
            ReflectType::USound => ffi::ReflectionType::USound,
            ReflectType::Composite => ffi::ReflectionType::Composite,
        })
    }
    let result = panic::catch_unwind(|| {
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
        is_event,
        is_editor_component,
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

unsafe extern "C" fn allocate(size: usize, align: usize, ptr: *mut ffi::RustAlloc) -> u32 {
    use std::alloc::{alloc, Layout};
    let layout = Layout::from_size_align(size, align);
    match layout {
        Ok(layout) => {
            let alloc_ptr = alloc(layout);
            *ptr = ffi::RustAlloc {
                ptr: alloc_ptr,
                size,
                align,
            };

            1
        }
        Err(_) => 0,
    }
}
pub fn create_allocate_fns() -> ffi::AllocateFns {
    ffi::AllocateFns { allocate }
}

pub extern "C" fn tick(dt: f32) -> crate::ffi::ResultCode {
    let r = panic::catch_unwind(|| unsafe {
        UnrealCore::tick(&mut crate::module::MODULE.as_mut().unwrap().core, dt);
    });
    match r {
        Ok(_) => ffi::ResultCode::Success,
        Err(_) => ffi::ResultCode::Panic,
    }
}

pub extern "C" fn begin_play() -> ffi::ResultCode {
    let r = panic::catch_unwind(|| unsafe {
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

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub struct StartupStage;

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum CoreStage {
    Startup,
    RegisterEvent,
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
    pub fn register_on_hit(&mut self) {
        unsafe {
            (bindings().actor_fns.register_actor_on_hit)(self.actor.0);
        }
    }

    pub fn set_owner(&mut self, new_owner: Option<&Self>) {
        unsafe {
            let ptr = new_owner
                .map(|comp| comp.actor.0 as *const AActorOpaque)
                .unwrap_or(std::ptr::null());
            (bindings().actor_fns.set_owner)(self.actor.0, ptr);
        }
    }

    pub fn get_actor_name(&self) -> String {
        unsafe {
            let mut alloc = ffi::RustAlloc::empty();
            (bindings().actor_fns.get_actor_name)(self.actor.0, &mut alloc);
            let name = {
                let slice = std::slice::from_raw_parts(alloc.ptr, alloc.size);
                let name = std::str::from_utf8(slice).unwrap();
                name.to_string()
            };
            alloc.free();
            name
        }
    }

    pub fn get_parent(&self, api: &UnrealApi) -> Option<Entity> {
        self.get_parent_actor()
            .and_then(|actor| api.actor_to_entity.get(&actor).copied())
    }

    pub fn get_parent_actor(&self) -> Option<ActorPtr> {
        unsafe {
            let mut parent = std::ptr::null_mut();
            (bindings().actor_fns.get_parent_actor)(self.actor.0, &mut parent);

            if parent.is_null() {
                None
            } else {
                Some(ActorPtr(parent))
            }
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

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct ActorPtr(pub *mut AActorOpaque);
impl ActorPtr {
    pub fn get_actor_name(&self) -> String {
        unsafe {
            let mut alloc = ffi::RustAlloc::empty();
            (bindings().actor_fns.get_actor_name)(self.0, &mut alloc);
            let name = {
                let slice = std::slice::from_raw_parts(alloc.ptr, alloc.size);
                let name = std::str::from_utf8(slice).unwrap();
                name.to_string()
            };
            alloc.free();
            name
        }
    }
}
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

        (bindings().actor_fns.get_spatial_data)(
            actor.actor.0,
            &mut position,
            &mut rotation,
            &mut scale,
        );

        transform.position = position.into();
        transform.rotation = rotation.into();
        transform.scale = scale.into();
        assert!(!transform.is_nan());
    }
}

fn upload_transform_to_unreal(query: Query<(&ActorComponent, &TransformComponent)>) {
    for (actor, transform) in query.iter() {
        let is_moveable = unsafe { (bindings().actor_fns.is_moveable)(actor.actor.0) } > 0;
        if !is_moveable {
            continue;
        }
        assert!(!transform.is_nan());
        (bindings().actor_fns.set_spatial_data)(
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
#[derive(Debug)]
pub struct Despawn {
    pub entity: Entity,
}

impl Command for Despawn {
    fn write(self, world: &mut World) {
        world.despawn(self.entity);
        if let Some(mut api) = world.get_resource_mut::<UnrealApi>() {
            // If this entity had an actor, we will also remove it from the map. Otherwise
            // `actor_to_entity` will grow indefinitely
            if let Some(actor) = api.entity_to_actor.remove(&self.entity) {
                api.actor_to_entity.remove(&actor);
                unsafe {
                    (bindings().actor_fns.destroy_actor)(actor.0);
                }
            }
        }
    }
}

/// It can can that actors are destroyed inside unreal for example from the kill plane. We need to
/// make sure to unregister them, otherwise we will end up with a dangling pointer in Rust.
/// Here we actually despawn the whole entity instead of just removing the `ActorComponent` because
/// it would be strange to keep the rust entity part alive, if the actor has been removed.
fn process_actor_destroyed(
    mut api: ResMut<UnrealApi>,
    mut reader: EventReader<ActorDestroyEvent>,
    mut commands: Commands,
) {
    for event in reader.iter() {
        if let Some(entity) = api.actor_to_entity.remove(&event.actor) {
            commands.add(Despawn { entity });
        }
    }
}

fn process_actor_spawned(
    mut api: ResMut<UnrealApi>,
    mut reader: EventReader<ActorSpawnedEvent>,
    mut commands: Commands,
) {
    unsafe {
        if let Some(global) = crate::module::MODULE.as_mut() {
            for &ActorSpawnedEvent { actor } in reader.iter() {
                let mut entity_cmds = commands.spawn();

                let mut len = 0;
                (bindings().editor_component_fns.get_editor_components)(
                    actor.0,
                    std::ptr::null_mut(),
                    &mut len,
                );

                let mut uuids = vec![ffi::Uuid::default(); len];
                (bindings().editor_component_fns.get_editor_components)(
                    actor.0,
                    uuids.as_mut_ptr(),
                    &mut len,
                );
                // We might have gotten back fewer uuids, so we truncate
                uuids.truncate(len);

                // We register all the components that are on the actor in unreal and add
                // them to the entity
                for uuid in uuids {
                    let mut alloc = StrRustAlloc::empty();
                    (bindings()
                        .editor_component_fns
                        .get_serialized_json_component)(
                        actor.0, uuid, &mut alloc
                    );

                    let json = alloc.into_string();

                    let uuid = from_ffi_uuid(uuid);
                    if let Some(insert) = global
                        .core
                        .module
                        .reflection_registry
                        .insert_serialized_component
                        .get(&uuid)
                    {
                        insert.insert_serialized_component(&json, &mut entity_cmds);
                    }
                }

                let entity = entity_cmds
                    .insert_bundle((ActorComponent { actor }, TransformComponent::default()))
                    .id();

                // Create a physics component if the root component is a primitive
                // component
                // TODO: We probably should get ALL the primitive components as well
                let mut root_component = std::ptr::null_mut();
                (bindings().actor_fns.get_root_component)(actor.0, &mut root_component);

                if !root_component.is_null() {
                    let is_primitive =
                        (bindings().is_a)(root_component, ffi::UObjectType::UPrimtiveComponent)
                            == 1;

                    if is_primitive {
                        let physics_component =
                            PhysicsComponent::new(UnrealPtr::from_raw(root_component));
                        commands.entity(entity).insert(physics_component);
                    }
                }

                api.register_actor(actor, entity);

                // Update the `EntityComponent` with the entity id so we can easily access
                // it in blueprint etc
                (bindings().actor_fns.set_entity_for_actor)(
                    actor.0,
                    ffi::Entity {
                        id: entity.to_bits(),
                    },
                );
            }
        }
    }
}
