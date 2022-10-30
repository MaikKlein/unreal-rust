// Fill out your copyright notice in the Description page of Project Settings.

#pragma once

#include "CoreMinimal.h"
#include "RustProperty.h"
#include "UObject/Object.h"
#include "RustEvent.generated.h"

/**
 * 
 */
USTRUCT()

struct RUSTPLUGIN_API FRustEvent
{
	GENERATED_BODY()

	UPROPERTY(EditAnywhere, Category="Rust")
	FString Guid;

	UPROPERTY(EditAnywhere, Category="Rust")
	FDynamicRustComponent Event;
};
