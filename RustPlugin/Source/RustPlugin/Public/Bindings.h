#pragma once

#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

struct AActorOpaque {
  uint8_t _inner[0];
};

struct Vector3 {
  float x;
  float y;
  float z;
};

struct Quaternion {
  float w;
  float x;
  float y;
  float z;
};

using GetSpatialDataFn = void(*)(const AActorOpaque *actor, Vector3 *position, Quaternion *rotation, Vector3 *scale);

using SetSpatialDataFn = void(*)(AActorOpaque *actor, Vector3 position, Quaternion rotation, Vector3 scale);

using LogFn = void(*)(const char*);

struct UnrealBindings {
  GetSpatialDataFn get_spatial_data;
  SetSpatialDataFn set_spatial_data;
  LogFn log;
};

struct Entity {
  uint64_t id;
};

using RegisterActorFn = Entity(*)(const AActorOpaque *actor);

struct RustBindings {
  RegisterActorFn register_actor;
};

using EntryUnrealBindingsFn = void(*)(UnrealBindings bindings);

using EntryBeginPlayFn = void(*)();

extern "C" {

extern void SetSpatialData(AActorOpaque *actor,
                           Vector3 position,
                           Quaternion rotation,
                           Vector3 scale);

extern void GetSpatialData(const AActorOpaque *actor,
                           Vector3 *position,
                           Quaternion *rotation,
                           Vector3 *scale);

extern void Log(const char *s);

} // extern "C"
