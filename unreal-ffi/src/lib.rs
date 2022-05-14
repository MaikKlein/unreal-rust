use glam::{Quat, Vec3};
use std::{ffi::c_void, os::raw::c_char};

#[repr(u8)]
#[derive(Debug)]
pub enum ResultCode {
    Success = 0,
    Panic = 1,
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone)]
pub struct Quaternion {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Entity {
    pub id: u64,
}
#[repr(C)]
#[derive(Default, Debug, Copy, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
impl Color {
    pub const RED: Self = Self {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const GREEN: Self = Self {
        r: 0,
        g: 255,
        b: 0,
        a: 255,
    };
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone)]
pub struct Movement {
    pub velocity: Vector3,
    pub is_falling: u32,
}

impl From<Quat> for Quaternion {
    fn from(v: Quat) -> Self {
        Quaternion {
            x: v.x,
            y: v.y,
            z: v.z,
            w: v.w,
        }
    }
}
impl From<Quaternion> for Quat {
    fn from(val: Quaternion) -> Self {
        Quat::from_xyzw(val.x, val.y, val.z, val.w)
    }
}

impl From<Vector3> for Vec3 {
    fn from(val: Vector3) -> Self {
        Vec3::new(val.x, val.y, val.z)
    }
}

impl From<Vec3> for Vector3 {
    fn from(v: Vec3) -> Self {
        Vector3 {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

#[repr(C)]
pub struct LineTraceParams {
    pub ignored_actors: *const *mut AActorOpaque,
    pub ignored_actors_len: usize,
}

// TODO: Is there a more typesafe way of defining an opaque type that
// is c ffi safe in Rust without nightly?
pub type AActorOpaque = c_void;
pub type UPrimtiveOpaque = c_void;
pub type UCapsuleOpaque = c_void;
pub type UClassOpague = c_void;
pub type FCollisionShapeOpague = c_void;

pub type GetSpatialDataFn = extern "C" fn(
    actor: *const AActorOpaque,
    position: &mut Vector3,
    rotation: &mut Quaternion,
    scale: &mut Vector3,
);

pub type LogFn = extern "C" fn(*const c_char, i32);

pub type SetSpatialDataFn = extern "C" fn(
    actor: *mut AActorOpaque,
    position: Vector3,
    rotation: Quaternion,
    scale: Vector3,
);
pub type IterateActorsFn = unsafe extern "C" fn(array: *mut *mut AActorOpaque, len: *mut u64);
pub type GetActionStateFn =
    unsafe extern "C" fn(name: *const c_char, len: usize, state: &mut ActionState);
pub type GetAxisValueFn = unsafe extern "C" fn(name: *const c_char, len: usize, value: &mut f32);
pub type SetEntityForActorFn = unsafe extern "C" fn(name: *mut AActorOpaque, entity: Entity);
pub type SpawnActorFn = unsafe extern "C" fn(
    actor_class: ActorClass,
    position: Vector3,
    rotation: Quaternion,
    scale: Vector3,
) -> *mut AActorOpaque;
pub type SetViewTargetFn = unsafe extern "C" fn(actor: *const AActorOpaque);
pub type GetMouseDeltaFn = unsafe extern "C" fn(x: &mut f32, y: &mut f32);
pub type GetActorComponentsFn =
    unsafe extern "C" fn(actor: *const AActorOpaque, data: *mut ActorComponentPtr, len: &mut usize);
pub type VisualLogSegmentFn =
    unsafe extern "C" fn(owner: *const AActorOpaque, start: Vector3, end: Vector3, color: Color);
pub type GetRootComponentFn =
    unsafe extern "C" fn(actor: *const AActorOpaque, data: *mut ActorComponentPtr);
pub type VisualLogCapsuleFn = unsafe extern "C" fn(
    owner: *const AActorOpaque,
    position: Vector3,
    rotation: Quaternion,
    half_height: f32,
    radius: f32,
    color: Color,
);
pub type GetRegisteredClassesFn =
    unsafe extern "C" fn(classes: *mut *mut UClassOpague, len: *mut usize);

pub type GetClassFn = unsafe extern "C" fn(actor: *const AActorOpaque) -> *mut UClassOpague;
pub type IsMoveableFn = unsafe extern "C" fn(actor: *const AActorOpaque) -> u32;
pub type GetActorNameFn =
    unsafe extern "C" fn(actor: *const AActorOpaque, data: *mut c_char, len: *mut usize);

extern "C" {
    pub fn SetSpatialData(
        actor: *mut AActorOpaque,
        position: Vector3,
        rotation: Quaternion,
        scale: Vector3,
    );

    pub fn GetSpatialData(
        actor: *const AActorOpaque,
        position: &mut Vector3,
        rotation: &mut Quaternion,
        scale: &mut Vector3,
    );
    pub fn TickActor(actor: *mut AActorOpaque, dt: f32);
    pub fn Log(s: *const c_char, len: i32);
    pub fn IterateActors(array: *mut *mut AActorOpaque, len: *mut u64);
    pub fn GetActionState(name: *const c_char, len: usize, state: &mut ActionState);
    pub fn GetAxisValue(name: *const c_char, len: usize, value: &mut f32);
    pub fn SetEntityForActor(name: *mut AActorOpaque, entity: Entity);
    pub fn SpawnActor(
        actor_class: ActorClass,
        position: Vector3,
        rotation: Quaternion,
        scale: Vector3,
    ) -> *mut AActorOpaque;
    pub fn SetViewTarget(actor: *const AActorOpaque);
    pub fn GetMouseDelta(x: &mut f32, y: &mut f32);
    pub fn GetActorComponents(
        actor: *const AActorOpaque,
        data: *mut ActorComponentPtr,
        len: &mut usize,
    );
    pub fn GetRootComponent(actor: *const AActorOpaque, data: *mut ActorComponentPtr);

    pub fn VisualLogSegment(owner: *const AActorOpaque, start: Vector3, end: Vector3, color: Color);
    pub fn VisualLogCapsule(
        owner: *const AActorOpaque,
        position: Vector3,
        rotation: Quaternion,
        half_height: f32,
        radius: f32,
        color: Color,
    );
    pub fn GetRegisteredClasses(classes: *mut *mut UClassOpague, len: *mut usize);
    pub fn GetClass(actor: *const AActorOpaque) -> *mut UClassOpague;
    pub fn IsMoveable(actor: *const AActorOpaque) -> u32;
    pub fn GetActorName(actor: *const AActorOpaque, data: *mut c_char, len: *mut usize);
}

#[repr(C)]
pub struct UnrealBindings {
    pub get_spatial_data: GetSpatialDataFn,
    pub set_spatial_data: SetSpatialDataFn,
    pub log: LogFn,
    pub iterate_actors: IterateActorsFn,
    pub get_action_state: GetActionStateFn,
    pub get_axis_value: GetAxisValueFn,
    pub set_entity_for_actor: SetEntityForActorFn,
    pub spawn_actor: SpawnActorFn,
    pub set_view_target: SetViewTargetFn,
    pub get_mouse_delta: GetMouseDeltaFn,
    pub get_actor_components: GetActorComponentsFn,
    pub visual_log_segment: VisualLogSegmentFn,
    pub visual_log_capsule: VisualLogCapsuleFn,
    pub physics_bindings: UnrealPhysicsBindings,
    pub get_root_component: GetRootComponentFn,
    pub get_registered_classes: GetRegisteredClassesFn,
    pub get_class: GetClassFn,
    pub is_moveable: IsMoveableFn,
    pub get_actor_name: GetActorNameFn,
}
unsafe impl Sync for UnrealBindings {}
unsafe impl Send for UnrealBindings {}

#[repr(u8)]
#[derive(Debug)]
pub enum ActionState {
    Pressed = 0,
    Released = 1,
    Held = 2,
    Nothing = 3,
}
#[repr(u32)]
#[derive(Debug)]
pub enum ActorClass {
    RustActor = 0,
    CameraActor = 1,
}
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ActorComponentType {
    Primitive,
}

#[repr(u8)]
#[derive(Debug)]
pub enum Mobility {
    Static = 0,
    Stationary = 1,
    Moveable = 2,
}

#[repr(C)]
#[derive(Debug)]
pub struct ActorComponentPtr {
    pub ty: ActorComponentType,
    pub ptr: *mut c_void,
}
impl Default for ActorComponentPtr {
    fn default() -> Self {
        Self {
            ty: ActorComponentType::Primitive,
            ptr: std::ptr::null_mut(),
        }
    }
}

#[repr(C)]
pub struct Uuid {
    pub bytes: [u8; 16],
}

pub type EntryUnrealBindingsFn = unsafe extern "C" fn(bindings: UnrealBindings) -> RustBindings;
pub type BeginPlayFn = unsafe extern "C" fn() -> ResultCode;
pub type TickFn = unsafe extern "C" fn(dt: f32) -> ResultCode;
pub type RetrieveUuids = unsafe extern "C" fn(ptr: *mut Uuid, len: *mut usize);
pub type GetVelocityRustFn =
    unsafe extern "C" fn(actor: *const AActorOpaque, velocity: &mut Vector3);

pub type CollisionShape = c_void;

#[repr(u32)]
pub enum EventType {
    ActorSpawned = 0,
}

#[repr(C)]
pub struct ActorSpawnedEvent {
    pub actor: *mut AActorOpaque,
}

#[repr(C)]
pub struct RustBindings {
    pub retrieve_uuids: RetrieveUuids,
    pub get_velocity: GetVelocityRustFn,
    pub tick: TickFn,
    pub begin_play: BeginPlayFn,
    pub unreal_event: UnrealEventFn,
    pub reflection_fns: ReflectionFns,
}
pub type UnrealEventFn = unsafe extern "C" fn(ty: *const EventType, data: *const c_void);
pub type GetVelocityFn = unsafe extern "C" fn(primitive: *const UPrimtiveOpaque) -> Vector3;
pub type SetVelocityFn = unsafe extern "C" fn(primitive: *mut UPrimtiveOpaque, velocity: Vector3);
pub type IsSimulatingFn = unsafe extern "C" fn(primitive: *const UPrimtiveOpaque) -> u32;
pub type AddForceFn = unsafe extern "C" fn(actor: *mut UPrimtiveOpaque, force: Vector3);
pub type AddImpulseFn = unsafe extern "C" fn(actor: *mut UPrimtiveOpaque, force: Vector3);
pub type LineTraceFn = unsafe extern "C" fn(
    start: Vector3,
    end: Vector3,
    params: LineTraceParams,
    result: &mut HitResult,
) -> u32;
pub type GetBoundingBoxExtentFn =
    unsafe extern "C" fn(primitive: *const UPrimtiveOpaque) -> Vector3;
pub type SweepFn = unsafe extern "C" fn(
    start: Vector3,
    end: Vector3,
    rotation: Quaternion,
    params: LineTraceParams,
    primitive: *const UPrimtiveOpaque,
    result: &mut HitResult,
) -> u32;

pub type OverlapMultiFn = unsafe extern "C" fn(
    shape: *mut FCollisionShapeOpague,
    position: Vector3,
    rotation: Quaternion,
    params: LineTraceParams,
    max_results: usize,
    result: *mut *mut OverlapResult,
) -> u32;
#[repr(C)]
pub struct UnrealPhysicsBindings {
    pub get_velocity: GetVelocityFn,
    pub set_velocity: SetVelocityFn,
    pub is_simulating: IsSimulatingFn,
    pub add_force: AddForceFn,
    pub add_impulse: AddImpulseFn,
    pub line_trace: LineTraceFn,
    pub get_bounding_box_extent: GetBoundingBoxExtentFn,
    pub sweep: SweepFn,
    pub overlap_multi: OverlapMultiFn,
}
#[repr(C)]
#[derive(Debug)]
pub struct HitResult {
    pub actor: *mut AActorOpaque,
    pub primtive: *mut UPrimtiveOpaque,
    pub distance: f32,
    pub normal: Vector3,
    pub location: Vector3,
    pub impact_location: Vector3,
    pub pentration_depth: f32,
}

impl Default for HitResult {
    fn default() -> Self {
        Self {
            actor: std::ptr::null_mut(),
            primtive: std::ptr::null_mut(),
            distance: Default::default(),
            normal: Default::default(),
            location: Default::default(),
            impact_location: Default::default(),
            pentration_depth: Default::default(),
        }
    }
}
#[repr(C)]
#[derive(Debug)]
pub struct OverlapResult {
    pub actor: *mut AActorOpaque,
    pub primtive: *mut UPrimtiveOpaque,
}
impl Default for OverlapResult {
    fn default() -> Self {
        Self {
            actor: std::ptr::null_mut(),
            primtive: std::ptr::null_mut(),
        }
    }
}

extern "C" {
    pub fn GetVelocity(primitive: *const UPrimtiveOpaque) -> Vector3;
    pub fn SetVelocity(primitive: *mut UPrimtiveOpaque, velocity: Vector3);
    pub fn IsSimulating(primitive: *const UPrimtiveOpaque) -> u32;
    pub fn AddForce(actor: *mut UPrimtiveOpaque, force: Vector3);
    pub fn AddImpulse(actor: *mut UPrimtiveOpaque, force: Vector3);
    pub fn LineTrace(
        start: Vector3,
        end: Vector3,
        params: LineTraceParams,
        result: &mut HitResult,
    ) -> u32;
    pub fn GetBoundingBoxExtent(primitive: *const UPrimtiveOpaque) -> Vector3;
    pub fn Sweep(
        start: Vector3,
        end: Vector3,
        rotation: Quaternion,
        params: LineTraceParams,
        primitive: *const UPrimtiveOpaque,
        result: &mut HitResult,
    ) -> u32;
    pub fn OverlapMulti(
        shape: *mut FCollisionShapeOpague,
        position: Vector3,
        rotation: Quaternion,
        params: LineTraceParams,
        max_results: usize,
        result: *mut *mut OverlapResult,
    ) -> u32;
}

#[repr(u32)]
pub enum ReflectionType {
    Float,
    Vector3,
    Bool,
}

pub type NumberOfFieldsFn = unsafe extern "C" fn(uuid: Uuid, out: *mut u32) -> u32;
pub type GetFieldNameFn = unsafe extern "C" fn(
    uuid: Uuid,
    field_idx: u32,
    name: *mut *const c_char,
    len: *mut usize,
) -> u32;
pub type GetFieldTypeFn = unsafe extern "C" fn(uuid: Uuid, field_idx: u32, ty: *mut ReflectionType) -> u32;

pub type GetFieldFloatValueFn =
    unsafe extern "C" fn(uuid: Uuid, entity: Entity, field_idx: u32, out: *mut f32) -> u32;
pub type GetFieldVector3ValueFn =
    unsafe extern "C" fn(uuid: Uuid, entity: Entity, field_idx: u32, out: *mut Vector3) -> u32;
pub type GetFieldBoolValueFn =
    unsafe extern "C" fn(uuid: Uuid, entity: Entity, field_idx: u32, out: *mut u32) -> u32;
#[repr(C)]
pub struct ReflectionFns {
    pub number_of_fields: NumberOfFieldsFn,
    pub get_field_type: GetFieldTypeFn,
    pub get_field_name: GetFieldNameFn,
    pub get_field_vector3_value: GetFieldVector3ValueFn,
    pub get_field_bool_value: GetFieldBoolValueFn,
    pub get_field_float_value: GetFieldFloatValueFn,
}
