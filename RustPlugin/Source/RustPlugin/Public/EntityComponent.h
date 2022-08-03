// Fill out your copyright notice in the Description page of Project Settings.

#pragma once

#include "CoreMinimal.h"
#include "Components/ActorComponent.h"
#include "Bindings.h"
#include "DetailWidgetRow.h"
#include "EntityComponent.generated.h"


DECLARE_DELEGATE_RetVal(FReply, FOnComponentRemoved);

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
class URustPropertyFloat : public URustProperty
{
	GENERATED_BODY()
public:
	UPROPERTY()
	float Data;
};

UCLASS()
class URustPropertyQuaternion : public URustProperty
{
	GENERATED_BODY()
public:
	UPROPERTY()
	FQuat Data;
};

UCLASS()
class UDynamicRustComponent : public UObject
{
	GENERATED_BODY()
public:
	void Initialize(FGuid Guid);
	UPROPERTY()
	TMap<FString, TObjectPtr<URustProperty>> Fields;
	UPROPERTY()
	FString Name;
	void Render(IDetailCategoryBuilder& DetailBuilder, FOnComponentRemoved OnComponentRemoved);
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
	UPROPERTY()
	TMap<FGuid, TObjectPtr<UDynamicRustComponent>> Components;

public:
	UFUNCTION(BlueprintCallable, Category="Rust|Utilities", meta=(Keywords = "entity"))
	virtual FEntity GetEntity();
};
