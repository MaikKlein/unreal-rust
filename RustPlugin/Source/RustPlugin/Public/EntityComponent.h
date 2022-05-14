// Fill out your copyright notice in the Description page of Project Settings.

#pragma once

#include "CoreMinimal.h"
#include "Components/ActorComponent.h"
#include "Bindings.h"
#include "EntityComponent.generated.h"


USTRUCT(BlueprintType)
struct FEntity {
	GENERATED_BODY()	
	uint64_t Id;
};
UCLASS(Blueprintable, meta=(BlueprintSpawnableComponent) )
class RUSTPLUGIN_API UEntityComponent : public UActorComponent
{
	GENERATED_BODY()

public:	
	// Sets default values for this component's properties
	UEntityComponent();
	Entity Id;

protected:
	// Called when the game starts
	virtual void BeginPlay() override;

public:	
	// Called every frame
	virtual void TickComponent(float DeltaTime, ELevelTick TickType, FActorComponentTickFunction* ThisTickFunction) override;
	UFUNCTION(BlueprintCallable, Category="Utilities", meta=(Keywords = "entity"))
	virtual FEntity GetEntity();
};
