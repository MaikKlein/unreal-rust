// Fill out your copyright notice in the Description page of Project Settings.

#pragma once

#include "CoreMinimal.h"
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

protected:
	// Called when the game starts or when spawned
	virtual void BeginPlay() override;

public:	
	// Called every frame
	virtual void Tick(float DeltaTime) override;
	UFUNCTION(BlueprintCallable, Category="Rust", meta=(DisplayName="Get Entity Velocity"))
	FVector GetRustVelocity();

};
