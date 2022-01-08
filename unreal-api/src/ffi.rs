//macro_rules! declare_fn {
//    (fn $name: ident [$($param: ident: $ty: ty),*]) => {
//        extern "C" {
//            pub fn $name(
//                $(
//                    $param: $ty,
//                )*
//            );
//        }
//        pub type $name = extern  "C" fn(
//            $(
//                $param: $ty,
//            )*
//        );
//
//    };
//}
use crate::math::{Quat, Vec3};
use std::os::raw::c_char;

#[repr(u8)]
#[derive(Debug)]
pub enum ResultCode {
    Success = 0,
    Panic = 1,
}

#[repr(C)]
#[derive(Default, Debug)]
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
#[derive(Default, Debug)]
pub struct Vector3 {
    x: f32,
    y: f32,
    z: f32,
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
impl Into<Quat> for Quaternion {
    fn into(self) -> Quat {
        Quat::from_xyzw(self.x, self.y, self.z, self.w)
    }
}

impl Into<Vec3> for Vector3 {
    fn into(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
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
pub struct AActorOpaque {
    _inner: [u8; 0],
}

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
pub type IterateActorsFn = extern "C" fn(array: *mut *mut AActorOpaque, len: *mut u64);

pub type GetActionStateFn = extern "C" fn(name: *const c_char, state: &mut ActionState);
pub type GetAxisValueFn = extern "C" fn(name: *const c_char, len: usize, value: &mut f32);
pub type SetEntityForActorFn = extern "C" fn(name: *mut AActorOpaque, entity: Entity);

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
    pub fn GetActionState(name: *const c_char, state: &mut ActionState);
    pub fn GetAxisValue(name: *const c_char, len: usize, value: &mut f32);
    pub fn SetEntityForActor(name: *mut AActorOpaque, entity: Entity);
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

pub extern "C" fn register_actor(actor: *const AActorOpaque) -> Entity {
    todo!()
}
pub type RegisterActorFn = extern "C" fn(actor: *const AActorOpaque) -> Entity;

#[repr(C)]
pub struct RustBindings {
    pub retrieve_uuids: RetrieveUuids,
}

#[repr(transparent)]
pub struct Uuid {
    pub bytes: [u8; 16]
}

pub type EntryUnrealBindingsFn = extern "C" fn(bindings: UnrealBindings) -> RustBindings;
pub type EntryBeginPlayFn = extern "C" fn() -> ResultCode;
pub type EntryTickFn = extern "C" fn(dt: f32) -> ResultCode;
pub type RetrieveUuids = unsafe extern "C" fn(ptr: *mut Uuid, len: *mut usize);
