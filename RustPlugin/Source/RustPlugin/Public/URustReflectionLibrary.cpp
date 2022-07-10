#include "URustReflectionLibrary.h"

#include "RustUtils.h"
#include "RustPlugin.h"

void URustReflectionLibrary::K2_GetReflectionVector3(UUuid* Id, FEntity EntityId, int32 Index, FVector& Out)
{
	if (Id == nullptr)
		return;

	auto Guid = ToFGuid(Id->Id);
	Vector3 V;
	Entity E;
	E.id = EntityId.Id;

	auto Module = GetModule();
	if (Module.Plugin.IsLoaded())
	{
		Module.Plugin.Rust.reflection_fns.get_field_vector3_value(Id->Id, E, Index, &V);
		Out = ToFVector(V);
	}
}

void URustReflectionLibrary::K2_GetReflectionBool(UUuid* Id, FEntity EntityId, int32 Index, bool& Out)
{
	if (Id == nullptr)
		return;

	uint32_t Result;
	Entity E;
	E.id = EntityId.Id;

	auto Module = GetModule();
	if (Module.Plugin.IsLoaded())
	{
		Module.Plugin.Rust.reflection_fns.get_field_bool_value(Id->Id, E, Index, &Result);
		Out = Result;
	}
}

void URustReflectionLibrary::K2_GetReflectionQuat(UUuid* Id, FEntity EntityId, int32 Index, FQuat& Out)
{
	if (Id == nullptr)
		return;

	Quaternion Result;
	Entity E;
	E.id = EntityId.Id;

	auto Module = GetModule();
	if (Module.Plugin.IsLoaded())
	{
		Module.Plugin.Rust.reflection_fns.get_field_quat_value(Id->Id, E, Index, &Result);
		Out = ToFQuat(Result);
	}
}

void URustReflectionLibrary::K2_GetReflectionFloat(UUuid* Id, FEntity EntityId, int32 Index, float& Out)
{
	if (Id == nullptr)
		return;

	float Result;
	Entity E;
	E.id = EntityId.Id;

	auto Module = GetModule();
	if (Module.Plugin.IsLoaded())
	{
		Module.Plugin.Rust.reflection_fns.get_field_float_value(Id->Id, E, Index, &Result);
		Out = Result;
	}
}
