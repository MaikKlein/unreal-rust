#include "URustReflectionLibrary.h"

#include "Api.h"
#include "RustPlugin.h"

void URustReflectionLibrary::K2_GetReflectionVector3(UUuid* Id, FEntity EntityId, int32 Index, FVector& Out)
{
	if(Id == nullptr)
		return;
	
	//auto Guid = ToFGuid(Id->Id);
	//UE_LOG(LogTemp, Warning, TEXT("Called Reflection %s %i %i"), *Guid.ToString(), EntityId.Id, Index);
	Vector3 V;
	Entity E;
	E.id = EntityId.Id;

	auto Module = GetModule();
	if(Module.Plugin.IsLoaded())
	{
		Module.Plugin.Rust.reflection_fns.get_field_vector3_value(Id->Id, E, Index, &V);
		Out = ToFVector(V);
		//UE_LOG(LogTemp, Warning, TEXT("Out %s"), *Out.ToString());
	}
}

void URustReflectionLibrary::K2_GetReflectionBool(UUuid* Id, FEntity EntityId, int32 Index, bool& Out)
{
	UE_LOG(LogTemp, Warning, TEXT("Bool Before Called Reflection %i %i"), EntityId.Id, Index);
	if(Id == nullptr)
		return;
	
	auto Guid = ToFGuid(Id->Id);
	UE_LOG(LogTemp, Warning, TEXT("Bool Called Reflection %s %i %i"), *Guid.ToString(), EntityId.Id, Index);
	Vector3 V;
	Entity E;
	E.id = EntityId.Id;

	auto Module = GetModule();
	if(Module.Plugin.IsLoaded())
	{
		//Module.Plugin.Rust.reflection_fns.get_field_vector3_value(Id->Id, E, Index, &V);
		//Out = ToFVector(V);
	}
}
