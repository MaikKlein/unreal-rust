// Fill out your copyright notice in the Description page of Project Settings.

#pragma once

#include "CoreMinimal.h"
#include "Bindings.h"
#include "DetailCategoryBuilder.h"
#include "Templates/SubclassOf.h"
#include "UObject/Object.h"
#include "RustProperty.generated.h"

class USoundBase;
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

// TODO: This is a disgusting hack. We store all the possible variants in this struct so that we can access them
// via the property system. The reason for that is `IPropertyHandle::SetValue` only implements a few overrides
// like for FVector, FRotator etc. It would have been nice if we could set values for non default types.
// There is `IPropertyHandle::AccessRawData` which we can use to write any data we want. But using this doesn't
// update all instances of this property. Eg if we edit the blueprint base class, but we already placed this class in
// a level, none of the properties would update for the blueprint classes that were already placed in the level.
// But `IPropertyHandle::SetValue` seems to support that.
USTRUCT()
struct FRustProperty
{
	GENERATED_BODY()

	//UPROPERTY(EditAnywhere)
	//TEnumAsByte<ERustPropertyTag> Tag;

	UPROPERTY(EditAnywhere, Category=Rust)
	int32 Tag;

	UPROPERTY(EditAnywhere, Category=Rust)
	float Float;

	UPROPERTY(EditAnywhere, Category=Rust)
	bool Bool;

	UPROPERTY(EditAnywhere, Category=Rust)
	FVector Vector;

	UPROPERTY(EditAnywhere, Category=Rust)
	FRotator Rotation;

	UPROPERTY(EditAnywhere, Category=Rust)
	TSubclassOf<AActor> Class;
	
	UPROPERTY(EditAnywhere, Category=Rust)
	TObjectPtr<USoundBase> Sound;
	static void Initialize(TSharedPtr<IPropertyHandle> Handle, ReflectionType Type);
};


USTRUCT()
struct FDynamicRustComponent
{
	GENERATED_BODY()

	UPROPERTY(EditAnywhere, Category=Rust)
	TMap<FString, FRustProperty> Fields;

	UPROPERTY(EditAnywhere, Category=Rust)
	FString Name;

	void Reload(TSharedPtr<IPropertyHandle> Handle, FGuid Guid);
	// Initializes the property handle. It sets the same, and adds all the fields to the hashmap.
	static void Initialize(TSharedPtr<IPropertyHandle> Handle, FGuid InitGuid);
	static void Render(TSharedRef<IPropertyHandle> MapHandle, IDetailCategoryBuilder& DetailBuilder,
	                   IDetailLayoutBuilder& LayoutBuilder);
};
