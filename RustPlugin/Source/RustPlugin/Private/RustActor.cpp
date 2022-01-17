// Fill out your copyright notice in the Description page of Project Settings.


#include "RustActor.h"
#include "RustPlugin.h"
#include "Api.h"

// Sets default values
ARustActor::ARustActor()
{
 	// Set this actor to call Tick() every frame.  You can turn this off to improve performance if you don't need it.
	PrimaryActorTick.bCanEverTick = true;

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

FVector ARustActor::GetRustVelocity() {
	Vector3 Velocity;
	GetModule().Plugin.Rust.get_velocity((const AActorOpaque*) this, &Velocity);
	return ToFVector(Velocity);
}