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
#include "RustUtils.h"

#define LOCTEXT_NAMESPACE "FRustPluginModule"

ARustGameModeBase::~ARustGameModeBase()
{
}

ARustGameModeBase::ARustGameModeBase()
{
	FRustPluginModule& LocalModule = FModuleManager::LoadModuleChecked<FRustPluginModule>(TEXT("RustPlugin"));
	LocalModule.GameMode = this;

	PrimaryActorTick.bStartWithTickEnabled = true;
	PrimaryActorTick.bCanEverTick = true;

	bBlockInput = false;
	AutoReceiveInput = EAutoReceiveInput::Player0;
	InputComponent = CreateDefaultSubobject<UInputComponent>(TEXT("InputComponent"));
	InputComponent->bBlockInput = bBlockInput;
	InputComponent->Priority = InputPriority;
	UInputDelegateBinding::BindInputDelegates(GetClass(), InputComponent);

	// PlayerInput = CreateDefaultSubobject<UPlayerInput>(TEXT("Input"));
}

void ARustGameModeBase::OnActorSpawnedHandler(AActor* actor)
{
	EventType Type = EventType::ActorSpawned;
	ActorSpawnedEvent Event;
	Event.actor = (AActorOpaque*)actor;
	GetRustModule().Plugin.Rust.unreal_event(&Type, (void*)&Event);
}

void ARustGameModeBase::OnActorBeginOverlap(AActor* OverlappedActor, AActor* OtherActor)
{
	EventType Type = EventType::ActorBeginOverlap;
	ActorBeginOverlap Event;
	Event.overlapped_actor = (AActorOpaque*)OverlappedActor;
	Event.other = (AActorOpaque*)OtherActor;
	GetRustModule().Plugin.Rust.unreal_event(&Type, (void*)&Event);
}

void ARustGameModeBase::OnActorEndOverlap(AActor* OverlappedActor, AActor* OtherActor)
{
	EventType Type = EventType::ActorEndOverlap;
	ActorEndOverlap Event;
	Event.overlapped_actor = (AActorOpaque*)OverlappedActor;
	Event.other = (AActorOpaque*)OtherActor;
	GetRustModule().Plugin.Rust.unreal_event(&Type, (void*)&Event);
}

void ARustGameModeBase::OnActorHit(AActor* SelfActor, AActor* OtherActor, FVector NormalImpulse, const FHitResult& Hit)
{
	EventType Type = EventType::ActorOnHit;
	ActorHitEvent Event;
	Event.self_actor = (AActorOpaque*)SelfActor;
	Event.other = (AActorOpaque*)OtherActor;
	Event.normal_impulse = ToVector3(NormalImpulse);
	GetRustModule().Plugin.Rust.unreal_event(&Type, (void*)&Event);
}

void ARustGameModeBase::OnActorDestroyed(AActor* Actor)
{
	EventType Type = EventType::ActorDestroy;
	ActorDestroyEvent Event;
	Event.actor = (AActorOpaque*)Actor;
	GetRustModule().Plugin.Rust.unreal_event(&Type, (void*)&Event);
}

void ARustGameModeBase::PostLogin(APlayerController* NewPlayer)
{
	Super::PostLogin(NewPlayer);
}

void ARustGameModeBase::Tick(float Dt)
{
	Super::Tick(Dt);
	FRustPluginModule& Module = GetRustModule();

	if (Module.Plugin.NeedsInit)
	{
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
	GetWorld()->AddOnActorSpawnedHandler(
		FOnActorSpawned::FDelegate::CreateUObject(this, &ARustGameModeBase::OnActorSpawnedHandler));

	APlayerController* PC = UGameplayStatics::GetPlayerController(this, 0);
	InputComponent->AxisBindings.Empty();
	for (auto Mapping : PC->PlayerInput->AxisMappings)
	{
		InputComponent->BindAxis(Mapping.AxisName);
	}

	FRustPluginModule& Module = GetRustModule();
	if (Module.Plugin.Rust.begin_play() == ResultCode::Panic)
	{
		Module.Exit();
	}
	else
	{
		Module.Plugin.NeedsInit = false;
	}
	for (TActorIterator<AActor> ActorItr(GetWorld()); ActorItr; ++ActorItr)
	{
		AActor* Actor = *ActorItr;
		Actor->OnDestroyed.AddUniqueDynamic(this, &ARustGameModeBase::OnActorDestroyed);
		OnActorSpawnedHandler(Actor);
	}
}
