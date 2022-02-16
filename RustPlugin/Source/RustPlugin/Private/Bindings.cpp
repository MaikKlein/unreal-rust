#include "Bindings.h"
#include "GameFramework/Actor.h"
#include "Api.h"
#include "Containers/UnrealString.h"
#include "RustActor.h"
#include "EngineUtils.h"
#include "RustPlugin.h"
#include "RustGameModeBase.h"
#include "Kismet/GameplayStatics.h"
#include "GameFramework/PlayerInput.h"
#include "EntityComponent.h"
#include "Camera/CameraActor.h"
#include "UObject/UObjectIterator.h"
#include "VisualLogger/VisualLogger.h"
#include "VisualLogger/VisualLoggerTypes.h"
#include "DrawDebugHelpers.h"

DEFINE_LOG_CATEGORY(RustVisualLog);

void SetSpatialData(AActorOpaque *actor,
                    const Vector3 position,
                    const Quaternion rotation,
                    const Vector3 scale)
{
    ToAActor(actor)->SetActorTransform(FTransform(ToFQuat(rotation), ToFVector(position), ToFVector(scale)));
}

void TickActor(const AActorOpaque *actor, float dt)
{
    ToAActor(actor)->Tick(dt);
}
void GetSpatialData(const AActorOpaque *actor,
                    Vector3 *position,
                    Quaternion *rotation,
                    Vector3 *scale)
{
    auto t = ToAActor(actor)->GetTransform();
    *position = ToVector3(t.GetTranslation());
    *rotation = ToQuaternion(t.GetRotation());
    *scale = ToVector3(t.GetScale3D());
}
void Log(const char *s, int32 len)
{
    // TODO: Can we get rid of that allocation?
    FString logString = FString(len, UTF8_TO_TCHAR(s));
    UE_LOG(LogTemp, Warning, TEXT("%s"), *logString);
}
void IterateActors(AActorOpaque **array, uint64_t *len)
{
    uint64_t i = 0;
    for (TActorIterator<ARustActor> ActorItr(GetModule().GameMode->GetWorld()); ActorItr; ++ActorItr, ++i)
    {
        if (i >= *len)
            return;
        AActorOpaque *a = (AActorOpaque *)*ActorItr;
        array[i] = a;
    }
    *len = i;
}
void GetActionState(const char *name, ActionState *state)
{

    APlayerController *PC = UGameplayStatics::GetPlayerController(GetModule().GameMode, 0);

    for (auto M : PC->PlayerInput->GetKeysForAction(name))
    {
        if (PC->PlayerInput->WasJustPressed(M.Key))
        {
            *state = ActionState::Pressed;
            return;
        }
        if (PC->PlayerInput->WasJustReleased(M.Key))
        {
            *state = ActionState::Released;
            return;
        }
        if (PC->PlayerInput->IsPressed(M.Key))
        {
            *state = ActionState::Held;
            return;
        }
    }
    *state = ActionState::Nothing;
}
void GetAxisValue(const char *name, uintptr_t len, float *value)
{
    FName AxisName((int32)len, name);
    *value = GetModule().GameMode->InputComponent->GetAxisValue(AxisName);
}

void SetEntityForActor(AActorOpaque *actor, Entity entity)
{
    UEntityComponent *Component = NewObject<UEntityComponent>(ToAActor(actor), TEXT("EntityComponent"));
    Component->Id = entity;
    Component->CreationMethod = EComponentCreationMethod::Native;
    Component->RegisterComponent();

    // auto id = ((UEntityComponent*)ToAActor(actor)->GetComponentByClass(UEntityComponent::StaticClass()))->Id;
    UE_LOG(LogTemp, Warning, TEXT("Entity with %i"), entity.id);
}
AActorOpaque *SpawnActor(ActorClass class_,
                         Vector3 position,
                         Quaternion rotation,
                         Vector3 scale)
{
    for (TObjectIterator<UClass> It; It; ++It)
    {

        UClass *Class = *It;

        FName Name = Class->ClassConfigName;
        if (Cast<ARustActor>(Class->GetDefaultObject(false)) != nullptr)
        {
            UE_LOG(LogTemp, Warning, TEXT("Class %s"), *Class->GetDesc());
        }
    }

    UClass *Class;
    switch (class_)
    {
    case ActorClass::CameraActor:
        Class = ACameraActor::StaticClass();
        break;
    case ActorClass::RustActor:
        Class = ARustActor::StaticClass();
        break;
    default:
        // :(
        Class = ARustActor::StaticClass();
    };
    FVector Pos = ToFVector(position);
    FRotator Rot = ToFQuat(rotation).Rotator();
    return (AActorOpaque *)GetModule().GameMode->GetWorld()->SpawnActor(Class, &Pos, &Rot, FActorSpawnParameters{});
}
void SetViewTarget(const AActorOpaque *actor)
{
    APlayerController *PC = UGameplayStatics::GetPlayerController(GetModule().GameMode, 0);
    PC->SetViewTarget(ToAActor(actor), FViewTargetTransitionParams());
}
void GetMouseDelta(float *x, float *y)
{
    APlayerController *PC = UGameplayStatics::GetPlayerController(GetModule().GameMode, 0);
    PC->GetInputMouseDelta(*x, *y);
}
void GetActorComponents(const AActorOpaque *actor, ActorComponentPtr *data, uintptr_t *len)
{
    TSet<UActorComponent *> Components = ToAActor(actor)->GetComponents();
    if (data == nullptr)
    {
        *len = Components.Num();
    }
    else
    {
        uintptr_t MaxComponentNum = *len;
        uintptr_t i = 0;
        for (auto &Component : Components)
        {
            if (i == MaxComponentNum)
            {
                break;
            }
            if (Cast<UPrimitiveComponent>(Component) != nullptr)
            {
                data[i] =
                    ActorComponentPtr{
                        ActorComponentType::Primitive,
                        (void *)Component};

                i += 1;
            }
        }
        *len = i;
    }
}
void AddForce(UPrimtiveOpaque *actor, Vector3 force)
{
    ((UPrimitiveComponent *)actor)->AddForce(ToFVector(force), FName{}, false);
}

void AddImpulse(UPrimtiveOpaque *actor, Vector3 force)
{
    ((UPrimitiveComponent *)actor)->AddImpulse(ToFVector(force), FName{}, false);
}
uint32_t IsSimulating(const UPrimtiveOpaque *primitive)
{
    return ((UPrimitiveComponent *)primitive)->IsSimulatingPhysics(FName{});
}
Vector3 GetVelocity(const UPrimtiveOpaque *primitive)
{
    return ToVector3(((UPrimitiveComponent *)primitive)->GetComponentVelocity());
}
void SetVelocity(UPrimtiveOpaque *primitive, Vector3 velocity)
{
    ((UPrimitiveComponent *)primitive)->SetPhysicsLinearVelocity(ToFVector(velocity), false, FName{});
}
uint32_t LineTrace(Vector3 start, Vector3 end, LineTraceParams Params, HitResult *result)
{

    FHitResult Out;
    auto CollisionParams = FCollisionQueryParams();
    for (uintptr_t i = 0; i < Params.ignored_actors_len; ++i)
    {
        CollisionParams.AddIgnoredActor((AActor *)Params.ignored_actors[i]);
    }
    bool IsHit = GetModule().GameMode->GetWorld()->LineTraceSingleByChannel(Out, ToFVector(start), ToFVector(end), ECollisionChannel::ECC_MAX, CollisionParams, FCollisionResponseParams{});
    if (IsHit)
    {
        result->actor = (AActorOpaque *)Out.GetActor();
        result->distance = Out.Distance;
        result->location = ToVector3(Out.Location);
        result->normal = ToVector3(Out.Normal);
        result->impact_location = ToVector3(Out.ImpactPoint);
        result->pentration_depth = Out.PenetrationDepth;
    }

    return IsHit;
}
void VisualLogSegment(const AActorOpaque *actor, Vector3 start, Vector3 end, Color color)
{
    UE_VLOG_SEGMENT(ToAActor(actor), RustVisualLog, Log, ToFVector(start), ToFVector(end), ToFColor(color), TEXT(""));
}
void VisualLogCapsule(const AActorOpaque *owner,
                      Vector3 position,
                      Quaternion rotation,
                      float half_height,
                      float radius,
                      Color color)
{

    DrawDebugCapsule(
        GetModule().GameMode->GetWorld(),
        ToFVector(position),
        half_height,
        radius,
        ToFQuat(rotation),
        ToFColor(color),
        false,
        0.0,
        1,
        1.0);
    UE_VLOG_CAPSULE(ToAActor(owner), RustVisualLog, Log, ToFVector(position), half_height, radius, ToFQuat(rotation), ToFColor(color), TEXT(""));
}
void GetRootComponent(const AActorOpaque *actor, ActorComponentPtr *data)
{
    USceneComponent *Root = ToAActor(actor)->GetRootComponent();

    if (Cast<UPrimitiveComponent>(Root) != nullptr)
    {
        *data = ActorComponentPtr{ActorComponentType::Primitive, (void *)Root};
        return;
    }
}
Vector3 GetBoundingBoxExtent(const UPrimtiveOpaque *primitive)
{
    return ToVector3(((UPrimitiveComponent *)primitive)->Bounds.BoxExtent);
}
uint32_t Sweep(Vector3 start,
               Vector3 end,
               Quaternion rotation,
               LineTraceParams params,
               const UPrimtiveOpaque *primitive,
               HitResult *result)
{
    FHitResult Out;
    auto CollisionParams = FCollisionQueryParams();
    for (uintptr_t i = 0; i < params.ignored_actors_len; ++i)
    {
        CollisionParams.AddIgnoredActor((AActor *)params.ignored_actors[i]);
        CollisionParams.bFindInitialOverlaps = true;
        CollisionParams.bDebugQuery = true;
    }
    bool IsHit = GetModule().GameMode->GetWorld()->SweepSingleByChannel(
        Out,
        ToFVector(start),
        ToFVector(end),
        ToFQuat(rotation),
        ECollisionChannel::ECC_MAX,
        ((UPrimitiveComponent *)primitive)->GetCollisionShape(),
        CollisionParams, FCollisionResponseParams{});
    if (IsHit)
    {
        result->actor = (AActorOpaque *)Out.GetActor();
        result->distance = Out.Distance;
        result->location = ToVector3(Out.Location);
        result->normal = ToVector3(Out.Normal);
        result->impact_location = ToVector3(Out.ImpactPoint);
        result->pentration_depth = Out.PenetrationDepth;
    }

    return IsHit;
}

void GetRegisteredClasses(UClassOpague **classes, uintptr_t *len)
{
    if (classes == nullptr)
    {
        *len = GetModule().GameMode->RegisteredClasses.Num();
        return;
    }
    auto GameMode = GetModule().GameMode;
    uintptr_t Count = *len;
    for (uintptr_t Idx = 0; Idx < Count; ++Idx)
    {
        classes[Idx] = (UClassOpague*)GameMode->RegisteredClasses[Idx].Get();
    }
}