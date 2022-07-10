// Fill out your copyright notice in the Description page of Project Settings.

#include "RustGameModeBase.h"
#include "Modules/ModuleManager.h"
#include "RustPlugin.h"
#include "EngineUtils.h"
#include "Utils.h"
#include "Components/InputComponent.h"
#include "GameFramework/InputSettings.h"
#include "Engine/InputDelegateBinding.h"
#include "GameFramework/PlayerInput.h"
#include "Kismet/GameplayStatics.h"
#include "Widgets/Notifications/SNotificationList.h"
#include "Framework/Notifications/NotificationManager.h"

#include "Editor.h"

#define LOCTEXT_NAMESPACE "FRustPluginModule"

ARustGameModeBase::~ARustGameModeBase()
{
}
ARustGameModeBase::ARustGameModeBase()
{
    PrimaryActorTick.bStartWithTickEnabled = true;
    PrimaryActorTick.bCanEverTick = true;

    FRustPluginModule &LocalModule = FModuleManager::LoadModuleChecked<FRustPluginModule>(TEXT("RustPlugin"));
    LocalModule.GameMode = this;

    bBlockInput = false;
    AutoReceiveInput = EAutoReceiveInput::Player0;
    InputComponent = CreateDefaultSubobject<UInputComponent>(TEXT("InputComponent"));
    // InputComponent->RegisterComponent();
    InputComponent->bBlockInput = bBlockInput;
    InputComponent->Priority = InputPriority;
    UInputDelegateBinding::BindInputDelegates(GetClass(), InputComponent);

    // PlayerInput = CreateDefaultSubobject<UPlayerInput>(TEXT("Input"));
}
void ARustGameModeBase::OnActorSpawnedHandler(AActor* actor){
    EventType Type = EventType::ActorSpawned;
    ActorSpawnedEvent Event;
    Event.actor = (AActorOpaque*) actor;
    GetModule().Plugin.Rust.unreal_event(&Type, (void*)&Event);
}

void ARustGameModeBase::PostLogin(APlayerController *NewPlayer)
{
    Super::PostLogin(NewPlayer);
}
void ARustGameModeBase::Tick(float Dt)
{
    Super::Tick(Dt);
    FRustPluginModule &Module = FModuleManager::LoadModuleChecked<FRustPluginModule>(TEXT("RustPlugin"));
    if (Module.ShouldReloadPlugin)
    {
        if (Module.Plugin.TryLoad())
        {
            Module.ShouldReloadPlugin = false;
            //FNotificationInfo Info(LOCTEXT("SpawnNotification_Notification", "Hotreload: Rust"));
            //Info.ExpireDuration = 2.0f;
            //FSlateNotificationManager::Get().AddNotification(Info);
        }
    }

    if (Module.Plugin.NeedsInit)
    {
        UE_LOG(LogTemp, Warning, TEXT("REINIT"));
        StartPlay();
    }
    if (Module.Plugin.Rust.tick(Dt) == ResultCode::Panic)
    {
        Module.Exit();
    }
}
void ARustGameModeBase::StartPlay()
{
    Super::StartPlay();
    GetWorld()->AddOnActorSpawnedHandler(FOnActorSpawned::FDelegate::CreateUObject(this, &ARustGameModeBase::OnActorSpawnedHandler));
    
    APlayerController *PC = UGameplayStatics::GetPlayerController(this, 0);
    InputComponent->AxisBindings.Empty();
    for (auto Mapping : PC->PlayerInput->AxisMappings)
    {
        InputComponent->BindAxis(Mapping.AxisName);
    }

    FRustPluginModule &Module = FModuleManager::LoadModuleChecked<FRustPluginModule>(TEXT("RustPlugin"));
    if (Module.Plugin.Rust.begin_play() == ResultCode::Panic)
    {
        Module.Exit();
    }
    else
    {
        Module.Plugin.NeedsInit = false;
    }
    for (TActorIterator<AActor> ActorItr(GetWorld()); ActorItr; ++ActorItr) {
        OnActorSpawnedHandler(*ActorItr);
    }
}