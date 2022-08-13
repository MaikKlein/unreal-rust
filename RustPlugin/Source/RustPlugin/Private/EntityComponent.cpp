// Fill out your copyright notice in the Description page of Project Settings.


#include "EntityComponent.h"

#include "RustProperty.h"
#include "Widgets/Input/NumericUnitTypeInterface.inl"



// Sets default values for this component's properties
UEntityComponent::UEntityComponent()
{
	PrimaryComponentTick.bCanEverTick = false;
}

FEntity UEntityComponent::GetEntity()
{
	return Id;
}



