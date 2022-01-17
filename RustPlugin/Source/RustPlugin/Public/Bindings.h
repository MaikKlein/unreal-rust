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

using GetSpatialDataFn = void(*)(const AActorOpaque *actor, Vector3 *position, Quaternion *rotation, Vector3 *scale);

using SetSpatialDataFn = void(*)(AActorOpaque *actor, Vector3 position, Quaternion rotation, Vector3 scale);

using LogFn = void(*)(const char*, int32_t);

using IterateActorsFn = void(*)(AActorOpaque **array, uint64_t *len);

using GetActionStateFn = void(*)(const char *name, ActionState *state);

using GetAxisValueFn = void(*)(const char *name, uintptr_t len, float *value);

using SetEntityForActorFn = void(*)(AActorOpaque *name, Entity entity);

using SpawnActorFn = AActorOpaque*(*)(ActorClass actor_class, Vector3 position, Quaternion rotation, Vector3 scale);

using SetViewTargetFn = void(*)(const AActorOpaque *actor);

using GetMouseDeltaFn = void(*)(float *x, float *y);

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
};

struct Uuid {
  uint8_t bytes[16];
};

using RetrieveUuids = void(*)(Uuid *ptr, uintptr_t *len);

using GetVelocityFn = void(*)(const AActorOpaque *actor, Vector3 *velocity);

using TickFn = ResultCode(*)(float dt);

using BeginPlayFn = ResultCode(*)();

struct RustBindings {
  RetrieveUuids retrieve_uuids;
  GetVelocityFn get_velocity;
  TickFn tick;
  BeginPlayFn begin_play;
};

using EntryUnrealBindingsFn = RustBindings(*)(UnrealBindings bindings);

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

extern void GetActionState(const char *name, ActionState *state);

extern void GetAxisValue(const char *name, uintptr_t len, float *value);

extern void SetEntityForActor(AActorOpaque *name, Entity entity);

extern AActorOpaque *SpawnActor(ActorClass actor_class,
                                Vector3 position,
                                Quaternion rotation,
                                Vector3 scale);

extern void SetViewTarget(const AActorOpaque *actor);

extern void GetMouseDelta(float *x, float *y);

} // extern "C"
