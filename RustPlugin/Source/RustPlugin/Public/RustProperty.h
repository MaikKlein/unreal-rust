// Fill out your copyright notice in the Description page of Project Settings.

#pragma once

#include "CoreMinimal.h"
#include "Bindings.h"
#include "DetailCategoryBuilder.h"
#include "UObject/Object.h"
#include "RustProperty.generated.h"

DECLARE_DELEGATE_RetVal(FReply, FOnComponentRemoved);

UENUM()
enum ERustPropertyTag
{
	Bool,
	Float,
	Vector,
	Quat,
	Class,
	Sound
};

USTRUCT()
struct FRustProperty
{
	GENERATED_BODY()

	//UPROPERTY(EditAnywhere)
	//TEnumAsByte<ERustPropertyTag> Tag;

	UPROPERTY(EditAnywhere)
	int32 Tag;

	UPROPERTY(EditAnywhere)
	float Float;

	UPROPERTY(EditAnywhere)
	bool Bool;

	UPROPERTY(EditAnywhere)
	FVector Vector;

	UPROPERTY(EditAnywhere)
	FRotator Rotation;

	UPROPERTY(EditAnywhere)
	TSubclassOf<AActor> Class;
	//TObjectPtr<UClass> Class;
	UPROPERTY(EditAnywhere)
	TObjectPtr<USoundBase> Sound;
	static void Initialize(TSharedPtr<IPropertyHandle> Handle, ReflectionType Type);
};


USTRUCT()
struct FDynamicRustComponent
{
	GENERATED_BODY()

	UPROPERTY(EditAnywhere)
	TMap<FString, FRustProperty> Fields;

	UPROPERTY(EditAnywhere)
	FString Name;

	void Reload(TSharedPtr<IPropertyHandle> Handle, FGuid Guid);
	static void Initialize(TSharedPtr<IPropertyHandle> Handle, FGuid InitGuid);
	static void Render(TSharedRef<IPropertyHandle> MapHandle, IDetailCategoryBuilder& DetailBuilder,
	                   IDetailLayoutBuilder& LayoutBuilder);
};
