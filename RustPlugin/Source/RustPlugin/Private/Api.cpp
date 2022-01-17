#include "Api.h"
#include "Modules/ModuleManager.h"
#include "RustPlugin.h"
#include "EngineUtils.h"

UnrealBindings CreateBindings() {
    UnrealBindings b;
    b.get_spatial_data = &GetSpatialData;
    b.set_spatial_data = &SetSpatialData;
    b.log = &Log;
    b.iterate_actors = &IterateActors;
    b.get_action_state = &GetActionState;
    b.get_axis_value = &GetAxisValue;
    b.set_entity_for_actor = &SetEntityForActor;
    b.spawn_actor = &SpawnActor;
    b.set_view_target = &SetViewTarget;
    b.get_mouse_delta = &GetMouseDelta;
    return b;
}

Quaternion ToQuaternion(FQuat q)
{
    Quaternion r;
    r.x = q.X;
    r.y = q.Y;
    r.z = q.Z;
    r.w = q.W;
    return r;
}

Vector3 ToVector3(FVector v)
{
    Vector3 r;
    r.x = v.X;
    r.y = v.Y;
    r.z = v.Z;
    return r;
}

FVector ToFVector(Vector3 v)
{
    return FVector(v.x, v.y, v.z);
}

FQuat ToFQuat(Quaternion q)
{
    return FQuat(q.x, q.y, q.z, q.w);
}

AActor *ToAActor(const AActorOpaque *actor)
{
    return (AActor *)actor;
}
AActor *ToAActor(AActorOpaque *actor)
{
    return (AActor *)actor;
}

FRustPluginModule& GetModule() {
    return FModuleManager::LoadModuleChecked<FRustPluginModule>(TEXT("RustPlugin"));
}