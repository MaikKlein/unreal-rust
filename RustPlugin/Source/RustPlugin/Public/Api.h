#pragma once

#include "CoreMinimal.h"
#include "Bindings.h"
#include "Engine/World.h"

class AActor;
class FRustPluginModule;

Quaternion ToQuaternion(FQuat q);

Vector3 ToVector3(FVector v);

FVector ToFVector(Vector3 v);

// W, X, Y, Z
FQuat ToFQuat(Quaternion q);

AActor *ToAActor(const AActorOpaque *actor);
AActor *ToAActor(AActorOpaque *actor);

UnrealBindings CreateBindings();
FRustPluginModule& GetModule();