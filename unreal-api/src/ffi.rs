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

#[repr(C)]
#[derive(Default)]
pub struct Quaternion {
    w: f32,
    x: f32,
    y: f32,
    z: f32,
}

#[repr(C)]
#[derive(Default)]
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
        Quat::from_xyzw(self.x,self.y,self.z, self.w)
    }
}

impl Into<Vec3> for Vector3 {
    fn into(self) -> Vec3 {
        Vec3::new(self.x,self.y,self.z)
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
pub struct Entity {
    pub id: u64,
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

pub type LogFn = extern "C" fn(*const c_char);

pub type SetSpatialDataFn = extern "C" fn(
    actor: *mut AActorOpaque,
    position: Vector3,
    rotation: Quaternion,
    scale: Vector3,
);

#[repr(C)]
pub struct UnrealBindings {
    pub get_spatial_data: GetSpatialDataFn,
    pub set_spatial_data: SetSpatialDataFn,
    pub log: LogFn,
}
unsafe impl Sync for UnrealBindings {}
unsafe impl Send for UnrealBindings {}

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
    pub fn Log(s: *const c_char);
}

pub extern "C" fn register_actor(actor: *const AActorOpaque) -> Entity {
    todo!()
}
pub type RegisterActorFn = extern "C" fn(actor: *const AActorOpaque) -> Entity;

#[repr(C)]
pub struct RustBindings {
    pub register_actor: RegisterActorFn,
}

#[no_mangle]
extern "C" fn create_rust_bindings() -> RustBindings {
    RustBindings {
        register_actor: register_actor,
    }
}
pub type CreateRustBindingsFn = extern "C" fn() -> RustBindings;
pub type EntryUnrealBindingsFn = extern "C" fn(bindings: UnrealBindings);
pub type EntryBeginPlayFn = extern "C" fn();
