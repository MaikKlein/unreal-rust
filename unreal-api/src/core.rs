use bevy_ecs::prelude::*;
use ffi::{ActorComponentPtr, ActorComponentType, EventType, Quaternion};
use std::{collections::HashMap, ffi::c_void, os::raw::c_char};

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
        world.insert_resource(Time::default());
        world.insert_resource(Input::default());
        world.insert_resource(ActorRegistration::default());

        let mut schedule = Schedule::default();
        schedule
            .add_stage(CoreStage::PreUpdate, SystemStage::single_threaded())
            .add_stage_after(
                CoreStage::PreUpdate,
                CoreStage::Update,
                SystemStage::single_threaded(),
            )
            .add_stage_after(
                CoreStage::Update,
                CoreStage::PostUpdate,
                SystemStage::single_threaded(),
            );

        schedule.add_system_to_stage(CoreStage::PreUpdate, update_input);
        schedule.add_system_to_stage(CoreStage::PreUpdate, download_transform_from_unreal);
        schedule.add_system_to_stage(CoreStage::PostUpdate, upload_transform_to_unreal);
        schedule.add_system_to_stage(CoreStage::PostUpdate, process_unreal_events);
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
        if let Some(mut time) = self.world.get_resource_mut::<Time>() {
            time.time += dt as f64;
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
        let reflect = global.core.reflection_registry.reflect.get(&uuid)?;

        let entity = Entity::from_bits(entity.id);
        reflect.get_field_value(&global.core.world, entity, idx)
    }
}

unsafe extern "C" fn number_of_fields(uuid: ffi::Uuid, out: *mut u32) -> u32 {
    fn get_number_fields(uuid: ffi::Uuid) -> Option<u32> {
        let global = unsafe { crate::module::MODULE.as_mut() }?;
        let uuid = from_ffi_uuid(uuid);
        let reflect = global.core.reflection_registry.reflect.get(&uuid)?;
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
        let reflect = global.core.reflection_registry.reflect.get(&uuid)?;
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

unsafe extern "C" fn get_field_name(
    uuid: ffi::Uuid,
    idx: u32,
    out: *mut *const c_char,
    len: *mut usize,
) -> u32 {
    fn get_field_name(uuid: ffi::Uuid, idx: u32) -> Option<&'static str> {
        let global = unsafe { crate::module::MODULE.as_mut() }?;
        let uuid = from_ffi_uuid(uuid);
        let reflect = global.core.reflection_registry.reflect.get(&uuid)?;
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
        let reflect = global.core.reflection_registry.reflect.get(&uuid)?;
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

use unreal_reflect::{
    registry::{ReflectType, ReflectValue, ReflectionRegistry},
    Component, Uuid,
};
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

#[derive(Copy, Clone)]
pub enum CollisionShape {
    Capsule { half_height: f32, radius: f32 },
    Box { half_extent: Vec3 },
    Sphere { radius: f32 },
}

impl CollisionShape {
    pub fn extent(self) -> Vec3 {
        match self {
            CollisionShape::Capsule {
                half_height,
                radius,
            } => Vec3::new(radius, radius, half_height),
            CollisionShape::Box { half_extent } => half_extent,
            CollisionShape::Sphere { radius } => Vec3::splat(radius),
        }
    }

    pub fn scale(self, amount: f32) -> Self {
        match self {
            CollisionShape::Capsule {
                half_height,
                radius,
            } => CollisionShape::Capsule {
                half_height: half_height * amount,
                radius: radius * amount,
            },
            CollisionShape::Box { half_extent } => CollisionShape::Box {
                half_extent: half_extent * amount,
            },
            CollisionShape::Sphere { radius } => CollisionShape::Sphere {
                radius: radius * amount,
            },
        }
    }

    pub fn inflate(self, amount: f32) -> Self {
        match self {
            CollisionShape::Capsule {
                half_height,
                radius,
            } => CollisionShape::Capsule {
                half_height: half_height + amount,
                radius: radius + amount,
            },
            CollisionShape::Box { half_extent } => CollisionShape::Box {
                half_extent: half_extent + amount,
            },
            CollisionShape::Sphere { radius } => CollisionShape::Sphere {
                radius: radius + amount,
            },
        }
    }
}

impl From<CollisionShape> for ffi::CollisionShape {
    fn from(val: CollisionShape) -> Self {
        match val {
            CollisionShape::Box { half_extent } => ffi::CollisionShape {
                ty: ffi::CollisionShapeType::Box,
                data: ffi::CollisionShapeUnion {
                    collision_box: ffi::CollisionBox {
                        half_extent_x: half_extent.x,
                        half_extent_y: half_extent.y,
                        half_extent_z: half_extent.z,
                    },
                },
            },
            CollisionShape::Capsule {
                half_height,
                radius,
            } => ffi::CollisionShape {
                data: ffi::CollisionShapeUnion {
                    capsule: ffi::CollisionCapsule {
                        radius,
                        half_height,
                    },
                },
                ty: ffi::CollisionShapeType::Capsule,
            },
            CollisionShape::Sphere { radius } => ffi::CollisionShape {
                data: ffi::CollisionShapeUnion {
                    sphere: ffi::CollisionSphere { radius },
                },
                ty: ffi::CollisionShapeType::Sphere,
            },
        }
    }
}

#[derive(Default, Component)]
#[uuid = "ffc10b5c-635c-43ce-8288-e3c6f6d67e36"]
pub struct PhysicsComponent {
    #[reflect(skip)]
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

    pub fn get_collision_shape(&self) -> CollisionShape {
        unsafe {
            let mut shape = ffi::CollisionShape::default();
            assert!(
                (bindings().physics_bindings.get_collision_shape)(self.ptr.ptr, &mut shape) == 1
            );
            match shape.ty {
                ffi::CollisionShapeType::Capsule => CollisionShape::Capsule {
                    half_height: shape.data.capsule.half_height,
                    radius: shape.data.capsule.radius,
                },
                ffi::CollisionShapeType::Box => CollisionShape::Box {
                    half_extent: Vec3::new(
                        shape.data.collision_box.half_extent_x,
                        shape.data.collision_box.half_extent_y,
                        shape.data.collision_box.half_extent_y,
                    ),
                },
                ffi::CollisionShapeType::Sphere => CollisionShape::Sphere {
                    radius: shape.data.sphere.radius,
                },
            }
        }
    }
    pub fn download_state(&mut self) {
        unsafe {
            self.is_simulating = (bindings().physics_bindings.is_simulating)(self.ptr.ptr) == 1;
            self.velocity = (bindings().physics_bindings.get_velocity)(self.ptr.ptr).into();
        }
    }

    pub fn upload_state(&mut self) {
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
