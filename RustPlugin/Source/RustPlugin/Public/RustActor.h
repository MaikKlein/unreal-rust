// Fill out your copyright notice in the Description page of Project Settings.

#pragma once

#include "CoreMinimal.h"
#include "EntityComponent.h"
#include "GameFramework/Actor.h"
#include "Misc/Guid.h"
#include "RustActor.generated.h"

UCLASS()
class ARustActor : public AActor
{
	GENERATED_BODY()

public:
	uint64 Entity;
	// Sets default values for this actor's properties
	ARustActor();
	UPROPERTY(Category=Rust, EditAnywhere)
	TArray<FGuid> Components;
	UPROPERTY(Category=Rust, EditAnywhere)
	UEntityComponent* EntityComponent;

protected:
	// Called when the game starts or when spawned
	virtual void BeginPlay() override;

public:
	// Called every frame
	virtual void Tick(float DeltaTime) override;
	UFUNCTION(BlueprintCallable, Category="Rust", meta=(DisplayName="Get Entity Component"))
	UEntityComponent* GetEntityComponent() { return EntityComponent; }
	UFUNCTION(BlueprintCallable, Category="Rust", meta=(DisplayName="Get Entity"))
	FEntity GetEntity()
	{
		if(EntityComponent != nullptr)
		{
			return EntityComponent->GetEntity();
		}

		return FEntity();
	}
};
