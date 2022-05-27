// Fill out your copyright notice in the Description page of Project Settings.


#include "EntityComponent.h"

// Sets default values for this component's properties
UEntityComponent::UEntityComponent()
{
	PrimaryComponentTick.bCanEverTick = false;
}

FEntity UEntityComponent::GetEntity() {
	return Id;
}

// Called when the game starts
void UEntityComponent::BeginPlay()
{
	
}


// Called every frame
void UEntityComponent::TickComponent(float DeltaTime, ELevelTick TickType, FActorComponentTickFunction* ThisTickFunction)
{
}

