#include "URustReflectionLibrary.h"

#include "Api.h"
#include "RustPlugin.h"

void URustReflectionLibrary::K2_GetReflectionVector3(UUuid* Id, FEntity EntityId, int32 Index, FVector& Out)
{
	UE_LOG(LogTemp, Warning, TEXT("Called Reflection"));
	Vector3 V;
	Entity E;
	E.id = EntityId.Id;
	GetModule().Plugin.Rust.reflection_fns.get_field_vector3_value(Id->Id, E, Index, &V);
	Out = ToFVector(V);
}
