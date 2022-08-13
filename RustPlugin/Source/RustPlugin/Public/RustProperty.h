// Fill out your copyright notice in the Description page of Project Settings.

#pragma once

#include "CoreMinimal.h"
#include "DetailCategoryBuilder.h"
#include "UObject/Object.h"
#include "RustProperty.generated.h"

class URustPropertyUClass;
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
class URustPropertyUClass : public URustProperty
{
	GENERATED_BODY()
public:
	UPROPERTY()
	UClass* Data;
};

UENUM()
enum ERustPropertyTag
{
	Bool,
	Float,
	Vector,
	Quat,
	Class,
};

USTRUCT()
struct FRustProperty2
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
	FQuat Rotation;

	UPROPERTY(EditAnywhere)
	TObjectPtr<UClass> Class;
};


USTRUCT()
struct FDynamicRustComponent2
{
	GENERATED_BODY()

	UPROPERTY(EditAnywhere)
	TMap<FString, FRustProperty2> Fields;

	UPROPERTY(EditAnywhere)
	FString Name;

	static void Initialize(TSharedPtr<IPropertyHandle> Handle, FGuid InitGuid);
	static void Render(TSharedRef<IPropertyHandle> MapHandle, IDetailCategoryBuilder& DetailBuilder,
	                   const TSharedRef<class IPropertyUtilities> Utilities, FOnComponentRemoved OnComponentRemoved);
};

UCLASS()
class UDynamicRustComponent : public UObject
{
	GENERATED_BODY()
public:
	void Initialize(FGuid Guid, UObject* Owner);
	UPROPERTY(VisibleAnywhere)
	TMap<FString, TObjectPtr<URustProperty>> Fields;
	UPROPERTY()
	FString Name;
	FGuid Guid;
	// TODO: Should not be here
	void Render(TSharedRef<IPropertyHandle> MapHandle, IDetailCategoryBuilder& DetailBuilder,
	            FOnComponentRemoved OnComponentRemoved);
};
