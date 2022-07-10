// Fill out your copyright notice in the Description page of Project Settings.


#include "RustActor.h"
#include "RustPlugin.h"
#include "Utils.h"

// Sets default values
ARustActor::ARustActor()
{
	// Set this actor to call Tick() every frame.  You can turn this off to improve performance if you don't need it.
	PrimaryActorTick.bCanEverTick = true;
	EntityComponent = CreateDefaultSubobject<UEntityComponent>(TEXT("EntityComponent"));
}

// Called when the game starts or when spawned
void ARustActor::BeginPlay()
{
	Super::BeginPlay();
}

// Called every frame
void ARustActor::Tick(float DeltaTime)
{
	Super::Tick(DeltaTime);
}