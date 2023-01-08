use std::os::raw::c_char;

use crate::{
    AActorOpaque, ActorComponentPtr, ActorSpawnOptions, Entity, Quaternion, RustAlloc,
    UClassOpague, USceneComponentOpague, UnrealTransform, Vector3,
};

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

pub type SetEntityForActorFn = unsafe extern "C" fn(name: *mut AActorOpaque, entity: Entity);

pub type GetActorComponentsFn =
    unsafe extern "C" fn(actor: *const AActorOpaque, data: *mut ActorComponentPtr, len: &mut usize);

pub type GetRootComponentFn =
    unsafe extern "C" fn(actor: *const AActorOpaque, data: *mut *mut USceneComponentOpague);

pub type GetRegisteredClassesFn =
    unsafe extern "C" fn(classes: *mut *mut UClassOpague, len: *mut usize);

pub type GetClassFn = unsafe extern "C" fn(actor: *const AActorOpaque) -> *mut UClassOpague;

pub type IsMoveableFn = unsafe extern "C" fn(actor: *const AActorOpaque) -> u32;

pub type GetActorNameFn = unsafe extern "C" fn(actor: *const AActorOpaque, data: *mut RustAlloc);

pub type SetOwnerFn =
    unsafe extern "C" fn(actor: *mut AActorOpaque, new_owner: *const AActorOpaque);

pub type RegisterActorOnOverlapFn = unsafe extern "C" fn(actor: *mut AActorOpaque);
pub type RegisterActorOnHitFn = unsafe extern "C" fn(actor: *mut AActorOpaque);

pub type SetViewTargetFn = unsafe extern "C" fn(actor: *const AActorOpaque);

pub type DestroyActorFn = unsafe extern "C" fn(actor: *const AActorOpaque);

pub type GetParentActorFn =
    unsafe extern "C" fn(actor: *const AActorOpaque, parent: *mut *mut AActorOpaque) -> u32;

pub type SpawnActorWithClassFn = unsafe extern "C" fn(
    actor_class: *const UClassOpague,
    transform: UnrealTransform,
    options: ActorSpawnOptions,
    out: *mut *mut AActorOpaque,
) -> u32;

extern "C" {
    pub fn RegisterActorOnHit(actor: *mut AActorOpaque);
    pub fn RegisterActorOnOverlap(actor: *mut AActorOpaque);

    pub fn SetOwner(actor: *mut AActorOpaque, new_owner: *const AActorOpaque);

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
    pub fn SetEntityForActor(name: *mut AActorOpaque, entity: Entity);

    pub fn GetActorComponents(
        actor: *const AActorOpaque,
        data: *mut ActorComponentPtr,
        len: &mut usize,
    );

    pub fn GetRootComponent(actor: *const AActorOpaque, data: *mut *mut USceneComponentOpague);

    pub fn GetRegisteredClasses(classes: *mut *mut UClassOpague, len: *mut usize);

    pub fn GetClass(actor: *const AActorOpaque) -> *mut UClassOpague;

    pub fn IsMoveable(actor: *const AActorOpaque) -> u32;

    pub fn GetActorName(actor: *const AActorOpaque, data: *mut RustAlloc);

    pub fn DestroyActor(actor: *const AActorOpaque);

    pub fn SetViewTarget(actor: *const AActorOpaque);

    pub fn GetParentActor(actor: *const AActorOpaque, parent: *mut *mut AActorOpaque) -> u32;

    pub fn SpawnActorWithClass(
        actor_class: *const UClassOpague,
        transform: UnrealTransform,
        options: ActorSpawnOptions,
        out: *mut *mut AActorOpaque,
    ) -> u32;
}

#[repr(C)]
pub struct ActorFns {
    pub get_spatial_data: GetSpatialDataFn,
    pub set_spatial_data: SetSpatialDataFn,
    pub set_entity_for_actor: SetEntityForActorFn,
    pub get_actor_components: GetActorComponentsFn,
    pub register_actor_on_overlap: RegisterActorOnOverlapFn,
    pub register_actor_on_hit: RegisterActorOnHitFn,
    pub get_root_component: GetRootComponentFn,
    pub get_registered_classes: GetRegisteredClassesFn,
    pub get_class: GetClassFn,
    pub set_view_target: SetViewTargetFn,
    pub get_actor_name: GetActorNameFn,
    pub set_owner: SetOwnerFn,
    pub is_moveable: IsMoveableFn,
    pub destroy_actor: DestroyActorFn,
    pub get_parent_actor: GetParentActorFn,
    pub spawn_actor_with_class: SpawnActorWithClassFn,
}
