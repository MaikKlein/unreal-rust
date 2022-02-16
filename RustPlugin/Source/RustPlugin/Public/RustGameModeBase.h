// Fill out your copyright notice in the Description page of Project Settings.

#pragma once

#include "CoreMinimal.h"
#include "GameFramework/GameModeBase.h"
#include "Containers/Map.h"
#include "RustGameModeBase.generated.h"

class FRustPluginModule;
class UPlayerInput;

struct FInputMap
{
	TMap<uint32, FName> AxisMapping;
	TMap<int32, FName> ActionMapping;
};
/**
 *
 */
UCLASS()
class RUSTPLUGIN_API ARustGameModeBase : public AGameModeBase
{
	GENERATED_BODY()
	ARustGameModeBase();
	~ARustGameModeBase();
	virtual void StartPlay();
	virtual void Tick(float Dt);
	UPlayerInput *PlayerInput;
	int32 Handle;
	virtual void PostLogin(APlayerController *NewPlayer);
	void OnActorSpawnedHandler(AActor *actor);

public:
	UPROPERTY(EditAnywhere, Category = Game)
	TArray<TSubclassOf<AActor>> RegisteredClasses;
};
