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

enum class ResultCode : uint8_t {
  Success = 0,
  Panic = 1,
};

struct AActorOpaque {
  uint8_t _inner[0];
};

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

using GetAxisValueFn = void(*)(const char *name, float *value);

using SetEntityForActorFn = void(*)(AActorOpaque *name, Entity entity);

struct UnrealBindings {
  GetSpatialDataFn get_spatial_data;
  SetSpatialDataFn set_spatial_data;
  LogFn log;
  IterateActorsFn iterate_actors;
  GetActionStateFn get_action_state;
  GetAxisValueFn get_axis_value;
  SetEntityForActorFn set_entity_for_actor;
};

using RegisterActorFn = Entity(*)(const AActorOpaque *actor);

struct RustBindings {
  RegisterActorFn register_actor;
};

using EntryUnrealBindingsFn = void(*)(UnrealBindings bindings);

using EntryBeginPlayFn = ResultCode(*)();

using EntryTickFn = ResultCode(*)(float dt);

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

extern void GetAxisValue(const char *name, float *value);

extern void SetEntityForActor(AActorOpaque *name, Entity entity);

} // extern "C"
