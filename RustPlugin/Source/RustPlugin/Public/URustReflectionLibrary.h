#pragma once

#include "EntityComponent.h"
#include "Kismet/BlueprintFunctionLibrary.h"
#include "URustReflectionLibrary.generated.h"

UCLASS()
class URustReflectionLibrary: public UBlueprintFunctionLibrary
{
	GENERATED_BODY()
public:

	UFUNCTION(BlueprintCallable, Category=Rust)
	static void K2_GetReflectionVector3(UUuid* Id, FEntity EntityId, int32 Index, FVector &Out);
	UFUNCTION(BlueprintCallable, Category=Rust)
	static void K2_GetReflectionBool(UUuid* Id, FEntity EntityId, int32 Index, bool &Out);
	UFUNCTION(BlueprintCallable, Category=Rust)
	static void K2_GetReflectionQuat(UUuid* Id, FEntity EntityId, int32 Index, FQuat &Out);
	UFUNCTION(BlueprintCallable, Category=Rust)
	static void K2_GetReflectionFloat(UUuid* Id, FEntity EntityId, int32 Index, float &Out);
	UFUNCTION(BlueprintCallable, Category=Rust)
	static bool K2_HasComponent(UUuid* Id, FEntity EntityId);
};

