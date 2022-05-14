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
	//GetModule().Plugin.Rust.get_velocity(static_cast<const AActorOpaque*>(this), &Velocity);

	if(EntityComponent != nullptr)
	{

		for(Uuid uuid: GetModule().Plugin.Uuids)
		{

			const char* Name;
			uintptr_t Len;
			if(GetModule().Plugin.Rust.reflection_fns.get_type_name(uuid, &Name, &Len))
			{
				FString N = FString(Len, UTF8_TO_TCHAR(Name));
				if(N == TEXT("MovementComponent"))
				{
					GetModule().Plugin.Rust.reflection_fns.get_field_vector3_value(uuid, this->EntityComponent->Id, 0, &Velocity);
					break;
				}
				//UE_LOG(LogTemp, Warning, TEXT("Code %s"), *N);
			}

			
		}
		//auto code = GetModule().Plugin.Rust.reflection_fns.get_field_vector3_value(Uuid, this->EntityComponent->Id, 0, &Velocity);
		//UE_LOG(LogTemp, Warning, TEXT("Code %i"), code);
	}
	
	return ToFVector(Velocity);
}