#pragma once

#include "CoreMinimal.h"
#include "Bindings.h"

class AActor;
class FRustPluginModule;

extern struct FLogCategoryRustVisualLog : public FLogCategory<ELogVerbosity::Log, ELogVerbosity::All>
{
	FORCEINLINE FLogCategoryRustVisualLog() : FLogCategory(TEXT("RustVisualLog"))
	{
	}
} RustVisualLog;;

Quaternion ToQuaternion(FQuat q);

Vector3 ToVector3(FVector v);

FVector ToFVector(Vector3 v);
FColor ToFColor(Color c);

// W, X, Y, Z
FQuat ToFQuat(Quaternion q);

AActor* ToAActor(const AActorOpaque* actor);
AActor* ToAActor(AActorOpaque* actor);

FGuid ToFGuid(Uuid uuid);
Uuid ToUuid(FGuid guid);

UnrealBindings CreateBindings();
FRustPluginModule& GetRustModule();

FCollisionShape ToFCollisionShape(CollisionShape Shape);


FString ToFString(Utf8Str Str);
struct FRustProperty* GetRustProperty(const AActorOpaque* actor, Uuid uuid, Utf8Str field);
