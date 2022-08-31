// Fill out your copyright notice in the Description page of Project Settings.


#include "EntityComponent.h"

#include "RustProperty.h"



// Sets default values for this component's properties
UEntityComponent::UEntityComponent()
{
	PrimaryComponentTick.bCanEverTick = false;
}

FEntity UEntityComponent::GetEntity()
{
	return Id;
}



