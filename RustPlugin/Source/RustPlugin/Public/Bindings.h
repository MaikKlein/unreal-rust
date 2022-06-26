#pragma once

#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

enum class ActionState : uint8_t {
  Pressed = 0,
  Released = 1,
  Held = 2,
  Nothing = 3,
};

enum class ActorClass : uint32_t {
  RustActor = 0,
  CameraActor = 1,
};

enum class ActorComponentType : uint32_t {
  Primitive,
};

enum class EventType : uint32_t {
  ActorSpawned = 0,
};

enum class ReflectionType : uint32_t {
  Float,
  Vector3,
  Bool,
  Quaternion,
  Composite,
};

enum class ResultCode : uint8_t {
  Success = 0,
  Panic = 1,
};

using AActorOpaque = void;

struct Vector3 {
  float x;
  float y;
  float z;
};

struct Quaternion {
  float x;
  float y;
  float z;
  float w;
};

struct Entity {
  uint64_t id;
};

struct ActorComponentPtr {
  ActorComponentType ty;
  void *ptr;
};

struct Color {
  uint8_t r;
  uint8_t g;
  uint8_t b;
  uint8_t a;
};
static const Color Color_RED = Color{ /* .r = */ 255, /* .g = */ 0, /* .b = */ 0, /* .a = */ 255 };
static const Color Color_GREEN = Color{ /* .r = */ 0, /* .g = */ 255, /* .b = */ 0, /* .a = */ 255 };

using UClassOpague = void;

using UPrimtiveOpaque = void;

struct LineTraceParams {
  AActorOpaque *const *ignored_actors;
  uintptr_t ignored_actors_len;
};

struct HitResult {
  AActorOpaque *actor;
  UPrimtiveOpaque *primtive;
  float distance;
  Vector3 normal;
  Vector3 location;
  Vector3 impact_location;
  float pentration_depth;
};

using FCollisionShapeOpague = void;

struct OverlapResult {
  AActorOpaque *actor;
  UPrimtiveOpaque *primtive;
};

using GetSpatialDataFn = void(*)(const AActorOpaque *actor, Vector3 *position, Quaternion *rotation, Vector3 *scale);

using SetSpatialDataFn = void(*)(AActorOpaque *actor, Vector3 position, Quaternion rotation, Vector3 scale);

using LogFn = void(*)(const char*, int32_t);

using IterateActorsFn = void(*)(AActorOpaque **array, uint64_t *len);

using GetActionStateFn = void(*)(const char *name, uintptr_t len, ActionState *state);

using GetAxisValueFn = void(*)(const char *name, uintptr_t len, float *value);

using SetEntityForActorFn = void(*)(AActorOpaque *name, Entity entity);

using SpawnActorFn = AActorOpaque*(*)(ActorClass actor_class, Vector3 position, Quaternion rotation, Vector3 scale);

using SetViewTargetFn = void(*)(const AActorOpaque *actor);

using GetMouseDeltaFn = void(*)(float *x, float *y);

using GetActorComponentsFn = void(*)(const AActorOpaque *actor, ActorComponentPtr *data, uintptr_t *len);

using VisualLogSegmentFn = void(*)(const AActorOpaque *owner, Vector3 start, Vector3 end, Color color);

using VisualLogCapsuleFn = void(*)(const AActorOpaque *owner, Vector3 position, Quaternion rotation, float half_height, float radius, Color color);

using GetVelocityFn = Vector3(*)(const UPrimtiveOpaque *primitive);

using SetVelocityFn = void(*)(UPrimtiveOpaque *primitive, Vector3 velocity);

using IsSimulatingFn = uint32_t(*)(const UPrimtiveOpaque *primitive);

using AddForceFn = void(*)(UPrimtiveOpaque *actor, Vector3 force);

using AddImpulseFn = void(*)(UPrimtiveOpaque *actor, Vector3 force);

using LineTraceFn = uint32_t(*)(Vector3 start, Vector3 end, LineTraceParams params, HitResult *result);

using GetBoundingBoxExtentFn = Vector3(*)(const UPrimtiveOpaque *primitive);

using SweepFn = uint32_t(*)(Vector3 start, Vector3 end, Quaternion rotation, LineTraceParams params, const UPrimtiveOpaque *primitive, HitResult *result);

using OverlapMultiFn = uint32_t(*)(FCollisionShapeOpague *shape, Vector3 position, Quaternion rotation, LineTraceParams params, uintptr_t max_results, OverlapResult **result);

struct UnrealPhysicsBindings {
  GetVelocityFn get_velocity;
  SetVelocityFn set_velocity;
  IsSimulatingFn is_simulating;
  AddForceFn add_force;
  AddImpulseFn add_impulse;
  LineTraceFn line_trace;
  GetBoundingBoxExtentFn get_bounding_box_extent;
  SweepFn sweep;
  OverlapMultiFn overlap_multi;
};

using GetRootComponentFn = void(*)(const AActorOpaque *actor, ActorComponentPtr *data);

using GetRegisteredClassesFn = void(*)(UClassOpague **classes, uintptr_t *len);

using GetClassFn = UClassOpague*(*)(const AActorOpaque *actor);

using IsMoveableFn = uint32_t(*)(const AActorOpaque *actor);

using GetActorNameFn = void(*)(const AActorOpaque *actor, char *data, uintptr_t *len);

struct UnrealBindings {
  GetSpatialDataFn get_spatial_data;
  SetSpatialDataFn set_spatial_data;
  LogFn log;
  IterateActorsFn iterate_actors;
  GetActionStateFn get_action_state;
  GetAxisValueFn get_axis_value;
  SetEntityForActorFn set_entity_for_actor;
  SpawnActorFn spawn_actor;
  SetViewTargetFn set_view_target;
  GetMouseDeltaFn get_mouse_delta;
  GetActorComponentsFn get_actor_components;
  VisualLogSegmentFn visual_log_segment;
  VisualLogCapsuleFn visual_log_capsule;
  UnrealPhysicsBindings physics_bindings;
  GetRootComponentFn get_root_component;
  GetRegisteredClassesFn get_registered_classes;
  GetClassFn get_class;
  IsMoveableFn is_moveable;
  GetActorNameFn get_actor_name;
};

struct Uuid {
  uint32_t a;
  uint32_t b;
  uint32_t c;
  uint32_t d;
};

using RetrieveUuids = void(*)(Uuid *ptr, uintptr_t *len);

using GetVelocityRustFn = void(*)(const AActorOpaque *actor, Vector3 *velocity);

using TickFn = ResultCode(*)(float dt);

using BeginPlayFn = ResultCode(*)();

using UnrealEventFn = void(*)(const EventType *ty, const void *data);

using NumberOfFieldsFn = uint32_t(*)(Uuid uuid, uint32_t *out);

using GetTypeNameFn = uint32_t(*)(Uuid uuid, const char **name, uintptr_t *len);

using GetFieldTypeFn = uint32_t(*)(Uuid uuid, uint32_t field_idx, ReflectionType *ty);

using GetFieldNameFn = uint32_t(*)(Uuid uuid, uint32_t field_idx, const char **name, uintptr_t *len);

using GetFieldVector3ValueFn = uint32_t(*)(Uuid uuid, Entity entity, uint32_t field_idx, Vector3 *out);

using GetFieldBoolValueFn = uint32_t(*)(Uuid uuid, Entity entity, uint32_t field_idx, uint32_t *out);

using GetFieldFloatValueFn = uint32_t(*)(Uuid uuid, Entity entity, uint32_t field_idx, float *out);

using GetFieldQuatValueFn = uint32_t(*)(Uuid uuid, Entity entity, uint32_t field_idx, Quaternion *out);

struct ReflectionFns {
  NumberOfFieldsFn number_of_fields;
  GetTypeNameFn get_type_name;
  GetFieldTypeFn get_field_type;
  GetFieldNameFn get_field_name;
  GetFieldVector3ValueFn get_field_vector3_value;
  GetFieldBoolValueFn get_field_bool_value;
  GetFieldFloatValueFn get_field_float_value;
  GetFieldQuatValueFn get_field_quat_value;
};

struct RustBindings {
  RetrieveUuids retrieve_uuids;
  GetVelocityRustFn get_velocity;
  TickFn tick;
  BeginPlayFn begin_play;
  UnrealEventFn unreal_event;
  ReflectionFns reflection_fns;
};

using EntryUnrealBindingsFn = RustBindings(*)(UnrealBindings bindings);

struct ActorSpawnedEvent {
  AActorOpaque *actor;
};

extern "C" {

extern void SetSpatialData(AActorOpaque *actor,
                           Vector3 position,
                           Quaternion rotation,
                           Vector3 scale);

extern void GetSpatialData(const AActorOpaque *actor,
                           Vector3 *position,
                           Quaternion *rotation,
                           Vector3 *scale);

extern void TickActor(AActorOpaque *actor, float dt);

extern void Log(const char *s, int32_t len);

extern void IterateActors(AActorOpaque **array, uint64_t *len);

extern void GetActionState(const char *name, uintptr_t len, ActionState *state);

extern void GetAxisValue(const char *name, uintptr_t len, float *value);

extern void SetEntityForActor(AActorOpaque *name, Entity entity);

extern AActorOpaque *SpawnActor(ActorClass actor_class,
                                Vector3 position,
                                Quaternion rotation,
                                Vector3 scale);

extern void SetViewTarget(const AActorOpaque *actor);

extern void GetMouseDelta(float *x, float *y);

extern void GetActorComponents(const AActorOpaque *actor, ActorComponentPtr *data, uintptr_t *len);

extern void GetRootComponent(const AActorOpaque *actor, ActorComponentPtr *data);

extern void VisualLogSegment(const AActorOpaque *owner, Vector3 start, Vector3 end, Color color);

extern void VisualLogCapsule(const AActorOpaque *owner,
                             Vector3 position,
                             Quaternion rotation,
                             float half_height,
                             float radius,
                             Color color);

extern void GetRegisteredClasses(UClassOpague **classes, uintptr_t *len);

extern UClassOpague *GetClass(const AActorOpaque *actor);

extern uint32_t IsMoveable(const AActorOpaque *actor);

extern void GetActorName(const AActorOpaque *actor, char *data, uintptr_t *len);

extern Vector3 GetVelocity(const UPrimtiveOpaque *primitive);

extern void SetVelocity(UPrimtiveOpaque *primitive, Vector3 velocity);

extern uint32_t IsSimulating(const UPrimtiveOpaque *primitive);

extern void AddForce(UPrimtiveOpaque *actor, Vector3 force);

extern void AddImpulse(UPrimtiveOpaque *actor, Vector3 force);

extern uint32_t LineTrace(Vector3 start, Vector3 end, LineTraceParams params, HitResult *result);

extern Vector3 GetBoundingBoxExtent(const UPrimtiveOpaque *primitive);

extern uint32_t Sweep(Vector3 start,
                      Vector3 end,
                      Quaternion rotation,
                      LineTraceParams params,
                      const UPrimtiveOpaque *primitive,
                      HitResult *result);

extern uint32_t OverlapMulti(FCollisionShapeOpague *shape,
                             Vector3 position,
                             Quaternion rotation,
                             LineTraceParams params,
                             uintptr_t max_results,
                             OverlapResult **result);

} // extern "C"
