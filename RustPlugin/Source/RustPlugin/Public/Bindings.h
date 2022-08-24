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
};

enum class ActorClass : uint32_t {
  RustActor = 0,
  CameraActor = 1,
};

enum class ActorComponentType : uint32_t {
  Primitive,
};

enum class CollisionShapeType : uint32_t {
  Box,
  Capsule,
  Sphere,
};

enum class EventType : uint32_t {
  ActorSpawned = 0,
  ActorBeginOverlap = 1,
  ActorEndOverlap = 2,
  ActorOnHit = 3,
};

enum class ReflectionType : uint32_t {
  Float,
  Vector3,
  Bool,
  Quaternion,
  UClass,
  USound,
  Composite,
};

enum class ResultCode : uint8_t {
  Success = 0,
  Panic = 1,
};

enum class UObjectType : uint32_t {
  UClass,
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

struct Color {
  uint8_t r;
  uint8_t g;
  uint8_t b;
  uint8_t a;
};

struct Utf8Str {
  const char *ptr;
  uintptr_t len;
};

struct Uuid {
  uint32_t a;
  uint32_t b;
  uint32_t c;
  uint32_t d;
};

using UObjectOpague = void;

using UOSoundBaseOpague = void;

struct SoundSettings {
  float volume;
  float pitch;
};

struct Entity {
  uint64_t id;
};

struct ActorComponentPtr {
  ActorComponentType ty;
  void *ptr;
};

using UClassOpague = void;

struct RustAlloc {
  uint8_t *ptr;
  uintptr_t size;
  uintptr_t align;
};

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
  Vector3 impact_normal;
  Vector3 impact_location;
  float pentration_depth;
  uint32_t start_penetrating;
};

struct CollisionBox {
  float half_extent_x;
  float half_extent_y;
  float half_extent_z;
};

struct CollisionSphere {
  float radius;
};

struct CollisionCapsule {
  float radius;
  float half_height;
};

union CollisionShapeUnion {
  CollisionBox collision_box;
  CollisionSphere sphere;
  CollisionCapsule capsule;
};

struct CollisionShape {
  CollisionShapeUnion data;
  CollisionShapeType ty;
};

struct OverlapResult {
  AActorOpaque *actor;
  UPrimtiveOpaque *primtive;
};

using GetSpatialDataFn = void(*)(const AActorOpaque *actor, Vector3 *position, Quaternion *rotation, Vector3 *scale);

using SetSpatialDataFn = void(*)(AActorOpaque *actor, Vector3 position, Quaternion rotation, Vector3 scale);

using SetEntityForActorFn = void(*)(AActorOpaque *name, Entity entity);

using GetActorComponentsFn = void(*)(const AActorOpaque *actor, ActorComponentPtr *data, uintptr_t *len);

using RegisterActorOnOverlapFn = void(*)(AActorOpaque *actor);

using RegisterActorOnHitFn = void(*)(AActorOpaque *actor);

using GetRootComponentFn = void(*)(const AActorOpaque *actor, ActorComponentPtr *data);

using GetRegisteredClassesFn = void(*)(UClassOpague **classes, uintptr_t *len);

using GetClassFn = UClassOpague*(*)(const AActorOpaque *actor);

using SetViewTargetFn = void(*)(const AActorOpaque *actor);

using GetActorNameFn = void(*)(const AActorOpaque *actor, RustAlloc *data);

using SetOwnerFn = void(*)(AActorOpaque *actor, const AActorOpaque *new_owner);

using IsMoveableFn = uint32_t(*)(const AActorOpaque *actor);

struct ActorFns {
  GetSpatialDataFn get_spatial_data;
  SetSpatialDataFn set_spatial_data;
  SetEntityForActorFn set_entity_for_actor;
  GetActorComponentsFn get_actor_components;
  RegisterActorOnOverlapFn register_actor_on_overlap;
  RegisterActorOnHitFn register_actor_on_hit;
  GetRootComponentFn get_root_component;
  GetRegisteredClassesFn get_registered_classes;
  GetClassFn get_class;
  SetViewTargetFn set_view_target;
  GetActorNameFn get_actor_name;
  SetOwnerFn set_owner;
  IsMoveableFn is_moveable;
};

using GetVelocityFn = Vector3(*)(const UPrimtiveOpaque *primitive);

using SetVelocityFn = void(*)(UPrimtiveOpaque *primitive, Vector3 velocity);

using IsSimulatingFn = uint32_t(*)(const UPrimtiveOpaque *primitive);

using AddForceFn = void(*)(UPrimtiveOpaque *actor, Vector3 force);

using AddImpulseFn = void(*)(UPrimtiveOpaque *actor, Vector3 force);

using LineTraceFn = uint32_t(*)(Vector3 start, Vector3 end, LineTraceParams params, HitResult *result);

using GetBoundingBoxExtentFn = Vector3(*)(const UPrimtiveOpaque *primitive);

using SweepFn = uint32_t(*)(Vector3 start, Vector3 end, Quaternion rotation, LineTraceParams params, CollisionShape collision_shape, HitResult *result);

using SweepMultiFn = uint32_t(*)(Vector3 start, Vector3 end, Quaternion rotation, LineTraceParams params, CollisionShape collision_shape, uintptr_t max_results, HitResult *results);

using OverlapMultiFn = uint32_t(*)(CollisionShape collision_shape, Vector3 position, Quaternion rotation, LineTraceParams params, uintptr_t max_results, OverlapResult **result);

using GetCollisionShapeFn = uint32_t(*)(const UPrimtiveOpaque *primitive, CollisionShape *shape);

struct PhysicsFns {
  GetVelocityFn get_velocity;
  SetVelocityFn set_velocity;
  IsSimulatingFn is_simulating;
  AddForceFn add_force;
  AddImpulseFn add_impulse;
  LineTraceFn line_trace;
  GetBoundingBoxExtentFn get_bounding_box_extent;
  SweepFn sweep;
  SweepMultiFn sweep_multi;
  OverlapMultiFn overlap_multi;
  GetCollisionShapeFn get_collision_shape;
};

using LogFn = void(*)(const char*, int32_t);

using IterateActorsFn = void(*)(AActorOpaque **array, uint64_t *len);

using GetActionStateFn = void(*)(const char *name, uintptr_t len, ActionState state, uint32_t *out);

using GetAxisValueFn = void(*)(const char *name, uintptr_t len, float *value);

using SpawnActorFn = AActorOpaque*(*)(ActorClass actor_class, Vector3 position, Quaternion rotation, Vector3 scale);

using GetMouseDeltaFn = void(*)(float *x, float *y);

using VisualLogSegmentFn = void(*)(const AActorOpaque *owner, Vector3 start, Vector3 end, Color color);

using VisualLogCapsuleFn = void(*)(Utf8Str category, const AActorOpaque *owner, Vector3 position, Quaternion rotation, float half_height, float radius, Color color);

using VisualLogLocationFn = void(*)(Utf8Str category, const AActorOpaque *owner, Vector3 position, float radius, Color color);

using GetEditorComponentUuidsFn = uint32_t(*)(const AActorOpaque *actor, Uuid *data, uintptr_t *len);

using GetEditorComponentQuatFn = uint32_t(*)(const AActorOpaque *actor, Uuid uuid, Utf8Str field, Quaternion *out);

using GetEditorComponentVectorFn = uint32_t(*)(const AActorOpaque *actor, Uuid uuid, Utf8Str field, Vector3 *out);

using GetEditorComponentBoolFn = uint32_t(*)(const AActorOpaque *actor, Uuid uuid, Utf8Str field, uint32_t *out);

using GetEditorComponentFloatFn = uint32_t(*)(const AActorOpaque *actor, Uuid uuid, Utf8Str field, float *out);

using GetEditorComponentUObjectFn = uint32_t(*)(const AActorOpaque *actor, Uuid uuid, Utf8Str field, UObjectType ty, UObjectOpague **out);

struct EditorComponentFns {
  GetEditorComponentUuidsFn get_editor_components;
  GetEditorComponentQuatFn get_editor_component_quat;
  GetEditorComponentVectorFn get_editor_component_vector;
  GetEditorComponentBoolFn get_editor_component_bool;
  GetEditorComponentFloatFn get_editor_component_float;
  GetEditorComponentUObjectFn get_editor_component_uobject;
};

using PlaySoundAtLocationFn = void(*)(const UOSoundBaseOpague *sound, Vector3 location, Quaternion rotation, const SoundSettings *settings);

struct SoundFns {
  PlaySoundAtLocationFn play_sound_at_location;
};

struct UnrealBindings {
  ActorFns actor_fns;
  PhysicsFns physics_fns;
  LogFn log;
  IterateActorsFn iterate_actors;
  GetActionStateFn get_action_state;
  GetAxisValueFn get_axis_value;
  SpawnActorFn spawn_actor;
  GetMouseDeltaFn get_mouse_delta;
  VisualLogSegmentFn visual_log_segment;
  VisualLogCapsuleFn visual_log_capsule;
  VisualLogLocationFn visual_log_location;
  EditorComponentFns editor_component_fns;
  SoundFns sound_fns;
};

using RetrieveUuids = void(*)(Uuid *ptr, uintptr_t *len);

using TickFn = ResultCode(*)(float dt);

using BeginPlayFn = ResultCode(*)();

using UnrealEventFn = void(*)(const EventType *ty, const void *data);

using NumberOfFieldsFn = uint32_t(*)(Uuid uuid, uint32_t *out);

using HasComponentFn = uint32_t(*)(Entity entity, Uuid uuid);

using GetTypeNameFn = uint32_t(*)(Uuid uuid, Utf8Str *name);

using GetFieldTypeFn = uint32_t(*)(Uuid uuid, uint32_t field_idx, ReflectionType *ty);

using GetFieldNameFn = uint32_t(*)(Uuid uuid, uint32_t field_idx, Utf8Str *name);

using GetFieldVector3ValueFn = uint32_t(*)(Uuid uuid, Entity entity, uint32_t field_idx, Vector3 *out);

using GetFieldBoolValueFn = uint32_t(*)(Uuid uuid, Entity entity, uint32_t field_idx, uint32_t *out);

using GetFieldFloatValueFn = uint32_t(*)(Uuid uuid, Entity entity, uint32_t field_idx, float *out);

using GetFieldQuatValueFn = uint32_t(*)(Uuid uuid, Entity entity, uint32_t field_idx, Quaternion *out);

struct ReflectionFns {
  NumberOfFieldsFn number_of_fields;
  HasComponentFn has_component;
  GetTypeNameFn get_type_name;
  GetFieldTypeFn get_field_type;
  GetFieldNameFn get_field_name;
  GetFieldVector3ValueFn get_field_vector3_value;
  GetFieldBoolValueFn get_field_bool_value;
  GetFieldFloatValueFn get_field_float_value;
  GetFieldQuatValueFn get_field_quat_value;
};

using AllocateFn = uint32_t(*)(uintptr_t size, uintptr_t align, RustAlloc *ptr);

struct AllocateFns {
  AllocateFn allocate;
};

struct RustBindings {
  RetrieveUuids retrieve_uuids;
  TickFn tick;
  BeginPlayFn begin_play;
  UnrealEventFn unreal_event;
  ReflectionFns reflection_fns;
  AllocateFns allocate_fns;
};

using EntryUnrealBindingsFn = uint32_t(*)(UnrealBindings bindings, RustBindings *rust_bindings);

struct ActorSpawnedEvent {
  AActorOpaque *actor;
};

struct ActorBeginOverlap {
  AActorOpaque *overlapped_actor;
  AActorOpaque *other;
};

struct ActorEndOverlap {
  AActorOpaque *overlapped_actor;
  AActorOpaque *other;
};

struct ActorHitEvent {
  AActorOpaque *self_actor;
  AActorOpaque *other;
  Vector3 normal_impulse;
};

extern "C" {

extern void TickActor(AActorOpaque *actor, float dt);

extern void Log(const char *s, int32_t len);

extern void IterateActors(AActorOpaque **array, uint64_t *len);

extern void GetActionState(const char *name, uintptr_t len, ActionState state, uint32_t *out);

extern void GetAxisValue(const char *name, uintptr_t len, float *value);

extern AActorOpaque *SpawnActor(ActorClass actor_class,
                                Vector3 position,
                                Quaternion rotation,
                                Vector3 scale);

extern void GetMouseDelta(float *x, float *y);

extern void VisualLogSegment(const AActorOpaque *owner, Vector3 start, Vector3 end, Color color);

extern void VisualLogCapsule(Utf8Str category,
                             const AActorOpaque *owner,
                             Vector3 position,
                             Quaternion rotation,
                             float half_height,
                             float radius,
                             Color color);

extern void VisualLogLocation(Utf8Str category,
                              const AActorOpaque *owner,
                              Vector3 position,
                              float radius,
                              Color color);

extern uint32_t GetEditorComponentUuids(const AActorOpaque *actor, Uuid *data, uintptr_t *len);

extern uint32_t GetEditorComponentVector(const AActorOpaque *actor,
                                         Uuid uuid,
                                         Utf8Str field,
                                         Vector3 *out);

extern uint32_t GetEditorComponentFloat(const AActorOpaque *actor,
                                        Uuid uuid,
                                        Utf8Str field,
                                        float *out);

extern uint32_t GetEditorComponentBool(const AActorOpaque *actor,
                                       Uuid uuid,
                                       Utf8Str field,
                                       uint32_t *out);

extern uint32_t GetEditorComponentQuat(const AActorOpaque *actor,
                                       Uuid uuid,
                                       Utf8Str field,
                                       Quaternion *out);

extern uint32_t GetEditorComponentUObject(const AActorOpaque *actor,
                                          Uuid uuid,
                                          Utf8Str field,
                                          UObjectType ty,
                                          UObjectOpague **out);

extern void PlaySoundAtLocation(const UOSoundBaseOpague *sound,
                                Vector3 location,
                                Quaternion rotation,
                                const SoundSettings *settings);

extern void RegisterActorOnHit(AActorOpaque *actor);

extern void RegisterActorOnOverlap(AActorOpaque *actor);

extern void SetOwner(AActorOpaque *actor, const AActorOpaque *new_owner);

extern void SetSpatialData(AActorOpaque *actor,
                           Vector3 position,
                           Quaternion rotation,
                           Vector3 scale);

extern void GetSpatialData(const AActorOpaque *actor,
                           Vector3 *position,
                           Quaternion *rotation,
                           Vector3 *scale);

extern void SetEntityForActor(AActorOpaque *name, Entity entity);

extern void GetActorComponents(const AActorOpaque *actor, ActorComponentPtr *data, uintptr_t *len);

extern void GetRootComponent(const AActorOpaque *actor, ActorComponentPtr *data);

extern void GetRegisteredClasses(UClassOpague **classes, uintptr_t *len);

extern UClassOpague *GetClass(const AActorOpaque *actor);

extern uint32_t IsMoveable(const AActorOpaque *actor);

extern void GetActorName(const AActorOpaque *actor, RustAlloc *data);

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
                      CollisionShape collision_shape,
                      HitResult *result);

extern uint32_t SweepMulti(Vector3 start,
                           Vector3 end,
                           Quaternion rotation,
                           LineTraceParams params,
                           CollisionShape collision_shape,
                           uintptr_t max_results,
                           HitResult *results);

extern uint32_t OverlapMulti(CollisionShape collision_shape,
                             Vector3 position,
                             Quaternion rotation,
                             LineTraceParams params,
                             uintptr_t max_results,
                             OverlapResult **result);

extern uint32_t GetCollisionShape(const UPrimtiveOpaque *primitive, CollisionShape *shape);

} // extern "C"
