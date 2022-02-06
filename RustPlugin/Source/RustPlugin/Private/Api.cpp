#include "Api.h"
#include "Modules/ModuleManager.h"
#include "RustPlugin.h"
#include "EngineUtils.h"

UnrealBindings CreateBindings()
{
    UnrealPhysicsBindings physics_bindings = {};
    physics_bindings.add_force = &AddForce;
    physics_bindings.add_impulse = &AddImpulse;
    physics_bindings.set_velocity = &SetVelocity;
    physics_bindings.get_velocity = &GetVelocity;
    physics_bindings.is_simulating = &IsSimulating;
    physics_bindings.line_trace = &LineTrace;
    physics_bindings.get_bounding_box_extent = &GetBoundingBoxExtent;
    physics_bindings.sweep = &Sweep;

    UnrealBindings b = {};
    b.physics_bindings = physics_bindings;
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
    b.get_actor_components = &GetActorComponents;
    b.visual_log_segment = &VisualLogSegment;
    b.visual_log_capsule = &VisualLogCapsule;
    b.get_root_component = &GetRootComponent;
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

FRustPluginModule &GetModule()
{
    return FModuleManager::LoadModuleChecked<FRustPluginModule>(TEXT("RustPlugin"));
}
FColor ToFColor(Color c)
{
    return FColor(c.r, c.g, c.b, c.a);
}