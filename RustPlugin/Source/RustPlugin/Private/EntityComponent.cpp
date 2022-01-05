// Fill out your copyright notice in the Description page of Project Settings.


#include "EntityComponent.h"

// Sets default values for this component's properties
UEntityComponent::UEntityComponent()
{
	PrimaryComponentTick.bCanEverTick = false;
}

FEntity UEntityComponent::GetEntity() {
	FEntity e;
	e.Id = (int64)Id.id;
	return e;
}
bool UEntityComponent::GetEntity2() const {
	return true;
}

// Called when the game starts
void UEntityComponent::BeginPlay()
{
	
}


// Called every frame
void UEntityComponent::TickComponent(float DeltaTime, ELevelTick TickType, FActorComponentTickFunction* ThisTickFunction)
{
}

