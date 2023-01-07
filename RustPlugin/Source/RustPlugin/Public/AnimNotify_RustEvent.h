// Fill out your copyright notice in the Description page of Project Settings.

#pragma once

#include "CoreMinimal.h"
#include "RustEvent.h"
#include "RustProperty.h"
#include "Animation/AnimNotifies/AnimNotify.h"
#include "AnimNotify_RustEvent.generated.h"

/**
 * 
 */
UCLASS()
class RUSTPLUGIN_API UAnimNotify_RustEvent : public UAnimNotify
{
	GENERATED_BODY()

public:
	UPROPERTY(EditAnywhere, Category="Rust")
	FRustEvent Event;

	virtual void Notify(USkeletalMeshComponent* MeshComp, UAnimSequenceBase* Animation,
	                    const FAnimNotifyEventReference& EventReference);
};
