use glam::{Quat, Vec3};
use std::{ffi::c_void, os::raw::c_char};
pub mod actor;
pub mod physics;

pub use actor::*;
pub use physics::*;

#[repr(u8)]
#[derive(Debug)]
pub enum ResultCode {
    Success = 0,
    Panic = 1,
}
#[repr(u32)]
#[derive(Copy, Clone)]
pub enum UObjectType {
    UClass,
}

#[repr(C)]
pub struct Utf8Str {
    pub ptr: *const c_char,
    pub len: usize,
}

impl<'a> From<&'a str> for Utf8Str {
    fn from(s: &'a str) -> Self {
        Self {
            ptr: s.as_ptr() as *const c_char,
            len: s.len(),
        }
    }
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

/// cbindgen:ignore
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
    pub const BLUE: Self = Self {
        r: 0,
        g: 0,
        b: 255,
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

// TODO: Is there a more typesafe way of defining an opaque type that
// is c ffi safe in Rust without nightly?
pub type AActorOpaque = c_void;
pub type UPrimtiveOpaque = c_void;
pub type UCapsuleOpaque = c_void;
pub type UClassOpague = c_void;
pub type UObjectOpague = c_void;
pub type UOSoundBaseOpague = c_void;

pub type LogFn = extern "C" fn(*const c_char, i32);
pub type IterateActorsFn = unsafe extern "C" fn(array: *mut *mut AActorOpaque, len: *mut u64);
pub type GetActionStateFn =
    unsafe extern "C" fn(name: *const c_char, len: usize, state: ActionState, out: *mut u32);
pub type GetAxisValueFn = unsafe extern "C" fn(name: *const c_char, len: usize, value: &mut f32);
pub type SpawnActorFn = unsafe extern "C" fn(
    actor_class: ActorClass,
    position: Vector3,
    rotation: Quaternion,
    scale: Vector3,
) -> *mut AActorOpaque;
pub type GetMouseDeltaFn = unsafe extern "C" fn(x: &mut f32, y: &mut f32);
pub type VisualLogSegmentFn =
    unsafe extern "C" fn(owner: *const AActorOpaque, start: Vector3, end: Vector3, color: Color);
pub type VisualLogCapsuleFn = unsafe extern "C" fn(
    category: Utf8Str,
    owner: *const AActorOpaque,
    position: Vector3,
    rotation: Quaternion,
    half_height: f32,
    radius: f32,
    color: Color,
);

pub type VisualLogLocationFn = unsafe extern "C" fn(
    category: Utf8Str,
    owner: *const AActorOpaque,
    position: Vector3,
    radius: f32,
    color: Color,
);

extern "C" {
    pub fn TickActor(actor: *mut AActorOpaque, dt: f32);
    pub fn Log(s: *const c_char, len: i32);
    pub fn IterateActors(array: *mut *mut AActorOpaque, len: *mut u64);
    pub fn GetActionState(name: *const c_char, len: usize, state: ActionState, out: *mut u32);
    pub fn GetAxisValue(name: *const c_char, len: usize, value: &mut f32);
    pub fn SpawnActor(
        actor_class: ActorClass,
        position: Vector3,
        rotation: Quaternion,
        scale: Vector3,
    ) -> *mut AActorOpaque;
    pub fn GetMouseDelta(x: &mut f32, y: &mut f32);

    pub fn VisualLogSegment(owner: *const AActorOpaque, start: Vector3, end: Vector3, color: Color);
    pub fn VisualLogCapsule(
        category: Utf8Str,
        owner: *const AActorOpaque,
        position: Vector3,
        rotation: Quaternion,
        half_height: f32,
        radius: f32,
        color: Color,
    );
    pub fn VisualLogLocation(
        category: Utf8Str,
        owner: *const AActorOpaque,
        position: Vector3,
        radius: f32,
        color: Color,
    );
}

#[repr(C)]
pub struct UnrealBindings {
    pub actor_fns: ActorFns,
    pub physics_fns: PhysicsFns,
    pub log: LogFn,
    pub iterate_actors: IterateActorsFn,
    pub get_action_state: GetActionStateFn,
    pub get_axis_value: GetAxisValueFn,
    pub spawn_actor: SpawnActorFn,
    pub get_mouse_delta: GetMouseDeltaFn,
    pub visual_log_segment: VisualLogSegmentFn,
    pub visual_log_capsule: VisualLogCapsuleFn,
    pub visual_log_location: VisualLogLocationFn,
    pub editor_component_fns: EditorComponentFns,
    pub sound_fns: SoundFns,
}
unsafe impl Sync for UnrealBindings {}
unsafe impl Send for UnrealBindings {}

#[repr(u8)]
#[derive(Debug)]
pub enum ActionState {
    Pressed = 0,
    Released = 1,
    Held = 2,
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
#[derive(Default, Clone, Copy)]
pub struct Uuid {
    pub a: u32,
    pub b: u32,
    pub c: u32,
    pub d: u32,
}

pub type EntryUnrealBindingsFn =
    unsafe extern "C" fn(bindings: UnrealBindings, rust_bindings: *mut RustBindings) -> u32;
pub type BeginPlayFn = unsafe extern "C" fn() -> ResultCode;
pub type TickFn = unsafe extern "C" fn(dt: f32) -> ResultCode;
pub type RetrieveUuids = unsafe extern "C" fn(ptr: *mut Uuid, len: *mut usize);
pub type GetVelocityRustFn =
    unsafe extern "C" fn(actor: *const AActorOpaque, velocity: &mut Vector3);

#[repr(u32)]
pub enum EventType {
    ActorSpawned = 0,
    ActorBeginOverlap = 1,
    ActorEndOverlap = 2,
}

#[repr(C)]
pub struct ActorSpawnedEvent {
    pub actor: *mut AActorOpaque,
}

#[repr(C)]
pub struct ActorBeginOverlap {
    pub overlapped_actor: *mut AActorOpaque,
    pub other: *mut AActorOpaque,
}
#[repr(C)]
pub struct ActorEndOverlap {
    pub overlapped_actor: *mut AActorOpaque,
    pub other: *mut AActorOpaque,
}

#[repr(C)]
pub struct RustBindings {
    pub retrieve_uuids: RetrieveUuids,
    pub tick: TickFn,
    pub begin_play: BeginPlayFn,
    pub unreal_event: UnrealEventFn,
    pub reflection_fns: ReflectionFns,
    pub allocate_fns: AllocateFns,
}
pub type UnrealEventFn = unsafe extern "C" fn(ty: *const EventType, data: *const c_void);

#[repr(u32)]
pub enum ReflectionType {
    Float,
    Vector3,
    Bool,
    Quaternion,
    UClass,
    USound,
    Composite,
}

pub type NumberOfFieldsFn = unsafe extern "C" fn(uuid: Uuid, out: *mut u32) -> u32;
pub type GetTypeNameFn = unsafe extern "C" fn(uuid: Uuid, name: *mut Utf8Str) -> u32;
pub type GetFieldNameFn =
    unsafe extern "C" fn(uuid: Uuid, field_idx: u32, name: *mut Utf8Str) -> u32;
pub type GetFieldTypeFn =
    unsafe extern "C" fn(uuid: Uuid, field_idx: u32, ty: *mut ReflectionType) -> u32;

pub type GetFieldFloatValueFn =
    unsafe extern "C" fn(uuid: Uuid, entity: Entity, field_idx: u32, out: *mut f32) -> u32;
pub type GetFieldVector3ValueFn =
    unsafe extern "C" fn(uuid: Uuid, entity: Entity, field_idx: u32, out: *mut Vector3) -> u32;
pub type GetFieldBoolValueFn =
    unsafe extern "C" fn(uuid: Uuid, entity: Entity, field_idx: u32, out: *mut u32) -> u32;
pub type GetFieldQuatValueFn =
    unsafe extern "C" fn(uuid: Uuid, entity: Entity, field_idx: u32, out: *mut Quaternion) -> u32;
pub type HasComponentFn = unsafe extern "C" fn(entity: Entity, uuid: Uuid) -> u32;

#[repr(C)]
pub struct ReflectionFns {
    pub number_of_fields: NumberOfFieldsFn,
    pub has_component: HasComponentFn,
    pub get_type_name: GetTypeNameFn,
    pub get_field_type: GetFieldTypeFn,
    pub get_field_name: GetFieldNameFn,
    pub get_field_vector3_value: GetFieldVector3ValueFn,
    pub get_field_bool_value: GetFieldBoolValueFn,
    pub get_field_float_value: GetFieldFloatValueFn,
    pub get_field_quat_value: GetFieldQuatValueFn,
}

#[repr(C)]
pub struct RustAlloc {
    pub ptr: *mut u8,
    pub size: usize,
    pub align: usize,
}

impl RustAlloc {
    pub fn empty() -> Self {
        Self {
            ptr: std::ptr::null_mut(),
            size: 0,
            align: 0,
        }
    }
    /// # Safety
    /// Must have a valid allocation from within unreal c++
    /// Only free if the ptr is not already in use
    /// Ptr must be valid, and allocated from the same allocator
    pub unsafe fn free(self) {
        if self.size == 0 || self.ptr.is_null() {
            return;
        }
        std::alloc::dealloc(
            self.ptr,
            std::alloc::Layout::from_size_align(self.size, self.align).unwrap(),
        );
    }
}

pub type AllocateFn = unsafe extern "C" fn(size: usize, align: usize, ptr: *mut RustAlloc) -> u32;

#[repr(C)]
pub struct AllocateFns {
    pub allocate: AllocateFn,
}

extern "C" {
    pub fn GetEditorComponentUuids(
        actor: *const AActorOpaque,
        data: *mut Uuid,
        len: *mut usize,
    ) -> u32;

    pub fn GetEditorComponentVector(
        actor: *const AActorOpaque,
        uuid: Uuid,
        field: Utf8Str,
        out: *mut Vector3,
    ) -> u32;
    pub fn GetEditorComponentFloat(
        actor: *const AActorOpaque,
        uuid: Uuid,
        field: Utf8Str,
        out: *mut f32,
    ) -> u32;
    pub fn GetEditorComponentBool(
        actor: *const AActorOpaque,
        uuid: Uuid,
        field: Utf8Str,
        out: *mut u32,
    ) -> u32;
    pub fn GetEditorComponentQuat(
        actor: *const AActorOpaque,
        uuid: Uuid,
        field: Utf8Str,
        out: *mut Quaternion,
    ) -> u32;
    pub fn GetEditorComponentUObject(
        actor: *const AActorOpaque,
        uuid: Uuid,
        field: Utf8Str,
        ty: UObjectType,
        out: *mut *mut UObjectOpague,
    ) -> u32;
}

pub type GetEditorComponentUuidsFn =
    unsafe extern "C" fn(actor: *const AActorOpaque, data: *mut Uuid, len: *mut usize) -> u32;

pub type GetEditorComponentQuatFn = unsafe extern "C" fn(
    actor: *const AActorOpaque,
    uuid: Uuid,
    field: Utf8Str,
    out: *mut Quaternion,
) -> u32;

pub type GetEditorComponentVectorFn = unsafe extern "C" fn(
    actor: *const AActorOpaque,
    uuid: Uuid,
    field: Utf8Str,
    out: *mut Vector3,
) -> u32;

pub type GetEditorComponentFloatFn = unsafe extern "C" fn(
    actor: *const AActorOpaque,
    uuid: Uuid,
    field: Utf8Str,
    out: *mut f32,
) -> u32;

pub type GetEditorComponentBoolFn = unsafe extern "C" fn(
    actor: *const AActorOpaque,
    uuid: Uuid,
    field: Utf8Str,
    out: *mut u32,
) -> u32;
pub type GetEditorComponentUObjectFn = unsafe extern "C" fn(
    actor: *const AActorOpaque,
    uuid: Uuid,
    field: Utf8Str,
    ty: UObjectType,
    out: *mut *mut UObjectOpague,
) -> u32;

#[repr(C)]
pub struct EditorComponentFns {
    pub get_editor_components: GetEditorComponentUuidsFn,
    pub get_editor_component_quat: GetEditorComponentQuatFn,
    pub get_editor_component_vector: GetEditorComponentVectorFn,
    pub get_editor_component_bool: GetEditorComponentBoolFn,
    pub get_editor_component_float: GetEditorComponentFloatFn,
    pub get_editor_component_uobject: GetEditorComponentUObjectFn,
}

#[repr(C)]
pub struct SoundSettings {
    pub volume: f32,
    pub pitch: f32,
}
impl Default for SoundSettings {
    fn default() -> Self {
        Self {
            volume: 1.0,
            pitch: 1.0,
        }
    }
}

extern "C" {
    pub fn PlaySoundAtLocation(
        sound: *const UOSoundBaseOpague,
        location: Vector3,
        rotation: Quaternion,
        settings: *const SoundSettings,
    );
}
pub type PlaySoundAtLocationFn = unsafe extern "C" fn(
    sound: *const UOSoundBaseOpague,
    location: Vector3,
    rotation: Quaternion,
    settings: *const SoundSettings,
);

#[repr(C)]
pub struct SoundFns {
    pub play_sound_at_location: PlaySoundAtLocationFn,
}
