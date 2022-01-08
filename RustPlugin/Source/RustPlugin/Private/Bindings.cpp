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
void IterateActors(AActorOpaque **array, uint64_t* len)
{
    uint64_t i = 0;
    for (TActorIterator<ARustActor> ActorItr(GetModule().GameMode->GetWorld()); ActorItr; ++ActorItr, ++i)
    {
        //APlayerController *PC = UGameplayStatics::GetPlayerController(GetModule().GameMode, 0);
        //PC->SetViewTarget(*ActorItr, FViewTargetTransitionParams());
        if(i >= *len) return;
        AActorOpaque* a = (AActorOpaque*) *ActorItr;
        array[i] = a;
    }
    *len = i;
}
void GetActionState(const char *name, ActionState *state) {
    
    APlayerController *PC = UGameplayStatics::GetPlayerController(GetModule().GameMode, 0);
    
    for(auto M : PC->PlayerInput->GetKeysForAction(name)) {
        if (PC->PlayerInput->WasJustPressed(M.Key)) {
            *state = ActionState::Pressed;
            return;
        }
        if (PC->PlayerInput->WasJustReleased(M.Key)) {
            *state = ActionState::Released;
            return;
        }
        if (PC->PlayerInput->IsPressed(M.Key)) {
            *state = ActionState::Held;
            return;
        }
    }
    *state = ActionState::Nothing;
}
void GetAxisValue(const char *name, uintptr_t len, float *value){
    FName AxisName((int32)len, name);
    *value = GetModule().GameMode->InputComponent->GetAxisValue(AxisName);
}

void SetEntityForActor(AActorOpaque * actor, Entity entity) {
    UEntityComponent* Component = NewObject<UEntityComponent>(ToAActor(actor), TEXT("EntityComponent"));
    Component->Id = entity;
    Component->CreationMethod = EComponentCreationMethod::Native;
    Component->RegisterComponent();

    //auto id = ((UEntityComponent*)ToAActor(actor)->GetComponentByClass(UEntityComponent::StaticClass()))->Id;
    UE_LOG(LogTemp, Warning, TEXT("Entity with %i"), entity.id);
}