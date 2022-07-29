// Fill out your copyright notice in the Description page of Project Settings.

#pragma once

#include "CoreMinimal.h"
#include "Components/ActorComponent.h"
#include "Bindings.h"
#include "EntityComponent.generated.h"


UCLASS()
class URustProperty : public UObject
{
	GENERATED_BODY()
};

UCLASS()
class URustPropertyVector : public URustProperty
{
	GENERATED_BODY()
public:
	UPROPERTY()
	FVector Data;
};

UCLASS()
class URustPropertyBool : public URustProperty
{
	GENERATED_BODY()
public:
	UPROPERTY()
	bool Data;
};

UCLASS()
class UDynamicRustComponent : public UObject
{
	GENERATED_BODY()
public:
	UPROPERTY()
	TMap<FString, URustProperty*> Fields;
};

USTRUCT(BlueprintType)
struct FEntity
{
	GENERATED_BODY()
	UPROPERTY(EditDefaultsOnly, Category=Rust)
	uint64 Id;

	Entity ToRustEntity()
	{
		Entity E;
		E.id = Id;
		return E;
	}
};

UCLASS(BlueprintType)
class UUuid : public UObject
{
	GENERATED_BODY()
public:
	Uuid Id;
};

UCLASS(Blueprintable, meta=(BlueprintSpawnableComponent))
class RUSTPLUGIN_API UEntityComponent : public UActorComponent
{
	GENERATED_BODY()

public:
	// Sets default values for this component's properties
	UEntityComponent();
	FEntity Id;
	UPROPERTY(Category=Rust, EditAnywhere)
	TMap<FGuid, UDynamicRustComponent*> Components;

public:
	// Called every frame
	virtual void TickComponent(float DeltaTime, ELevelTick TickType,
	                           FActorComponentTickFunction* ThisTickFunction) override;
	UFUNCTION(BlueprintCallable, Category="Rust|Utilities", meta=(Keywords = "entity"))
	virtual FEntity GetEntity();
};
