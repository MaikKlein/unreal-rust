#pragma once

#include "EntityComponent.h"
#include "URustReflectionLibrary.generated.h"

UCLASS()
class URustReflectionLibrary: public UBlueprintFunctionLibrary
{
	GENERATED_BODY()
public:

	UFUNCTION(BlueprintCallable)
	static void K2_GetReflectionVector3(UUuid* Id, FEntity EntityId, int32 Index, FVector &Out);
	UFUNCTION(BlueprintCallable)
	static void K2_GetReflectionBool(UUuid* Id, FEntity EntityId, int32 Index, bool &Out);
};
