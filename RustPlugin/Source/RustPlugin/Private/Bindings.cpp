#include "Bindings.h"
#include "GameFramework/Actor.h"
#include "RustUtils.h"
#include "Containers/UnrealString.h"
#include "RustActor.h"
#include "EngineUtils.h"
#include "RustPlugin.h"
#include "RustGameModeBase.h"
#include "Kismet/GameplayStatics.h"
#include "GameFramework/PlayerInput.h"
#include "EntityComponent.h"
#include "Camera/CameraActor.h"
#include "VisualLogger/VisualLogger.h"

DEFINE_LOG_CATEGORY(RustVisualLog);

void SetSpatialData(AActorOpaque* actor,
                    const Vector3 position,
                    const Quaternion rotation,
                    const Vector3 scale)
{
	ToAActor(actor)->SetActorTransform(FTransform(ToFQuat(rotation), ToFVector(position), ToFVector(scale)));
}

void TickActor(const AActorOpaque* actor, float dt)
{
	ToAActor(actor)->Tick(dt);
}

void GetSpatialData(const AActorOpaque* actor,
                    Vector3* position,
                    Quaternion* rotation,
                    Vector3* scale)
{
	auto t = ToAActor(actor)->GetTransform();
	*position = ToVector3(t.GetTranslation());
	*rotation = ToQuaternion(t.GetRotation());
	*scale = ToVector3(t.GetScale3D());
}

void Log(const char* s, int32 len)
{
	// TODO: Can we get rid of that allocation?
	FString LogString = FString(len, UTF8_TO_TCHAR(s));
	UE_LOG(LogTemp, Warning, TEXT("%s"), *LogString);
}

void IterateActors(AActorOpaque** array, uint64_t* len)
{
	uint64_t i = 0;
	for (TActorIterator<ARustActor> ActorItr(GetModule().GameMode->GetWorld()); ActorItr; ++ActorItr, ++i)
	{
		if (i >= *len)
			return;
		AActorOpaque* a = (AActorOpaque*)*ActorItr;
		array[i] = a;
	}
	*len = i;
}

void GetActionState(const char* name, uintptr_t len, ActionState* state)
{
	APlayerController* PC = UGameplayStatics::GetPlayerController(GetModule().GameMode, 0);

	FName ActionName((int32)len, name);

	// TODO: I think this logic is broken. I think we can have both a pressed and a released event
	// at the same time. If that happens we will only process the `Pressed` event.

	for (auto M : PC->PlayerInput->GetKeysForAction(ActionName))
	{
		if (PC->PlayerInput->WasJustPressed(M.Key))
		{
			*state = ActionState::Pressed;
			return;
		}
		if (PC->PlayerInput->WasJustReleased(M.Key))
		{
			*state = ActionState::Released;
			return;
		}
		if (PC->PlayerInput->IsPressed(M.Key))
		{
			*state = ActionState::Held;
			return;
		}
	}
	*state = ActionState::Nothing;
}

void GetAxisValue(const char* name, uintptr_t len, float* value)
{
	FName AxisName((int32)len, name);
	*value = GetModule().GameMode->InputComponent->GetAxisValue(AxisName);
}

void SetEntityForActor(AActorOpaque* actor, Entity entity)
{
	ARustActor* RustActor = Cast<ARustActor>(ToAActor(actor));
	if (RustActor != nullptr && RustActor->EntityComponent != nullptr)
	{
		RustActor->EntityComponent->Id.Id = entity.id;
	}
	else
	{
		AActor* Actor = ToAActor(actor);
		UEntityComponent* Component = NewObject<UEntityComponent>(ToAActor(actor), TEXT("EntityComponent"));
		Component->Id.Id = entity.id;


		//Actor->AttachToComponent(Actor->GetRootComponent(), FAttachmentTransformRules::KeepRelativeTransform);
		Component->CreationMethod = EComponentCreationMethod::Native;
		ToAActor(actor)->AddOwnedComponent(Component);
		Component->RegisterComponent();
		////Component->RegisterComponent();
	}
	// auto id = ((UEntityComponent*)ToAActor(actor)->GetComponentByClass(UEntityComponent::StaticClass()))->Id;
	//UE_LOG(LogTemp, Warning, TEXT("Entity with %i"), entity.id);
}

AActorOpaque* SpawnActor(ActorClass class_,
                         Vector3 position,
                         Quaternion rotation,
                         Vector3 scale)
{
	//for (TObjectIterator<UClass> It; It; ++It)
	//{
	//	UClass* Class = *It;

	//	FName Name = Class->ClassConfigName;
	//	if (Cast<ARustActor>(Class->GetDefaultObject(false)) != nullptr)
	//	{
	//		UE_LOG(LogTemp, Warning, TEXT("Class %s"), *Class->GetDesc());
	//	}
	//}

	UClass* Class;
	switch (class_)
	{
	case ActorClass::CameraActor:
		Class = ACameraActor::StaticClass();
		break;
	case ActorClass::RustActor:
		Class = ARustActor::StaticClass();
		break;
	default:
		// :(
		Class = ARustActor::StaticClass();
	};
	FVector Pos = ToFVector(position);
	FRotator Rot = ToFQuat(rotation).Rotator();
	return (AActorOpaque*)GetModule().GameMode->GetWorld()->SpawnActor(Class, &Pos, &Rot, FActorSpawnParameters{});
}

void SetViewTarget(const AActorOpaque* actor)
{
	APlayerController* PC = UGameplayStatics::GetPlayerController(GetModule().GameMode, 0);
	PC->SetViewTarget(ToAActor(actor), FViewTargetTransitionParams());
}

void GetMouseDelta(float* x, float* y)
{
	APlayerController* PC = UGameplayStatics::GetPlayerController(GetModule().GameMode, 0);
	PC->GetInputMouseDelta(*x, *y);
}

void GetActorComponents(const AActorOpaque* actor, ActorComponentPtr* data, uintptr_t* len)
{
	TSet<UActorComponent*> Components = ToAActor(actor)->GetComponents();
	if (data == nullptr)
	{
		*len = Components.Num();
	}
	else
	{
		uintptr_t MaxComponentNum = *len;
		uintptr_t i = 0;
		for (auto& Component : Components)
		{
			if (i == MaxComponentNum)
			{
				break;
			}
			if (Cast<UPrimitiveComponent>(Component) != nullptr)
			{
				data[i] =
					ActorComponentPtr{
						ActorComponentType::Primitive,
						(void*)Component
					};

				i += 1;
			}
		}
		*len = i;
	}
}

void AddForce(UPrimtiveOpaque* actor, Vector3 force)
{
	((UPrimitiveComponent*)actor)->AddForce(ToFVector(force), FName{}, false);
}

void AddImpulse(UPrimtiveOpaque* actor, Vector3 force)
{
	((UPrimitiveComponent*)actor)->AddImpulse(ToFVector(force), FName{}, false);
}

uint32_t IsSimulating(const UPrimtiveOpaque* primitive)
{
	return ((UPrimitiveComponent*)primitive)->IsSimulatingPhysics(FName{});
}

Vector3 GetVelocity(const UPrimtiveOpaque* primitive)
{
	return ToVector3(((UPrimitiveComponent*)primitive)->GetComponentVelocity());
}

void SetVelocity(UPrimtiveOpaque* primitive, Vector3 velocity)
{
	((UPrimitiveComponent*)primitive)->SetPhysicsLinearVelocity(ToFVector(velocity), false, FName{});
}

uint32_t LineTrace(Vector3 start, Vector3 end, LineTraceParams Params, HitResult* result)
{
	FHitResult Out;
	auto CollisionParams = FCollisionQueryParams();
	for (uintptr_t i = 0; i < Params.ignored_actors_len; ++i)
	{
		CollisionParams.AddIgnoredActor((AActor*)Params.ignored_actors[i]);
	}
	bool IsHit = GetModule().GameMode->GetWorld()->LineTraceSingleByChannel(
		Out, ToFVector(start), ToFVector(end), ECollisionChannel::ECC_MAX, CollisionParams, FCollisionResponseParams{});
	if (IsHit)
	{
		result->actor = (AActorOpaque*)Out.GetActor();
		result->primtive = (UPrimtiveOpaque*)Out.GetComponent();
		result->distance = Out.Distance;
		result->location = ToVector3(Out.Location);
		result->normal = ToVector3(Out.Normal);
		result->impact_location = ToVector3(Out.ImpactPoint);
		result->pentration_depth = Out.PenetrationDepth;
	}

	return IsHit;
}

uint32_t OverlapMulti(CollisionShape shape,
                      Vector3 position,
                      Quaternion rotation,
                      LineTraceParams params,
                      uintptr_t max_results,
                      OverlapResult** results)
{
	TArray<FOverlapResult> Out;
	auto CollisionParams = FCollisionQueryParams();
	for (uintptr_t i = 0; i < params.ignored_actors_len; ++i)
	{
		CollisionParams.AddIgnoredActor((AActor*)params.ignored_actors[i]);
	}
	bool IsHit = GetModule().GameMode->GetWorld()->OverlapMultiByChannel(Out,
	                                                                     ToFVector(position),
	                                                                     ToFQuat(rotation),
	                                                                     ECollisionChannel::ECC_MAX,
	                                                                     ToFCollisionShape(shape),
	                                                                     CollisionParams,
	                                                                     FCollisionResponseParams{});
	if (IsHit)
	{
		uintptr_t Length = FGenericPlatformMath::Min(max_results, (uintptr_t)Out.Num());
		for (uintptr_t i = 0; i < Length; ++i)
		{
			OverlapResult* result = results[i];
			FOverlapResult* Hit = &Out[i];
			result->actor = (AActorOpaque*)Hit->GetActor();
			result->primtive = (UPrimtiveOpaque*)Hit->GetComponent();
		}
	}

	return IsHit;
}

void VisualLogSegment(const AActorOpaque* actor, Vector3 start, Vector3 end, Color color)
{
	UE_VLOG_SEGMENT(ToAActor(actor), RustVisualLog, Log, ToFVector(start), ToFVector(end), ToFColor(color), TEXT(""));
}

void VisualLogCapsule(
	Utf8Str category,
	const AActorOpaque* owner,
	Vector3 position,
	Quaternion rotation,
	float half_height,
	float radius,
	Color color)
{
	//DrawDebugCapsule(
	//	GetModule().GameMode->GetWorld(),
	//	ToFVector(position),
	//	half_height,
	//	radius,
	//	ToFQuat(rotation),
	//	ToFColor(color),
	//	false,
	//	0.0,
	//	1,
	//	1.0);
	FString CategoryStr = ToFString(category);
	auto LogCat = FLogCategory<ELogVerbosity::Log, ELogVerbosity::All>(*CategoryStr);
	FVector Base = ToFVector(position) - half_height * (ToFQuat(rotation) * FVector::UpVector);
	UE_VLOG_CAPSULE(ToAActor(owner), LogCat, Log, Base, half_height, radius, ToFQuat(rotation),
	                ToFColor(color), TEXT(""));
}

void GetRootComponent(const AActorOpaque* actor, ActorComponentPtr* data)
{
	USceneComponent* Root = ToAActor(actor)->GetRootComponent();

	if (Cast<UPrimitiveComponent>(Root) != nullptr)
	{
		*data = ActorComponentPtr{ActorComponentType::Primitive, (void*)Root};
		return;
	}
}

Vector3 GetBoundingBoxExtent(const UPrimtiveOpaque* primitive)
{
	return ToVector3(((UPrimitiveComponent*)primitive)->Bounds.BoxExtent);
}

uint32_t Sweep(Vector3 start,
               Vector3 end,
               Quaternion rotation,
               LineTraceParams params,
               CollisionShape shape,
               HitResult* result)
{
	FHitResult Out;
	auto CollisionParams = FCollisionQueryParams();
	for (uintptr_t i = 0; i < params.ignored_actors_len; ++i)
	{
		CollisionParams.AddIgnoredActor((AActor*)params.ignored_actors[i]);
		CollisionParams.bFindInitialOverlaps = true;
		// TODO: Make configurable
		CollisionParams.bDebugQuery = true;
	}
	bool IsHit = GetModule().GameMode->GetWorld()->SweepSingleByChannel(
		Out,
		ToFVector(start),
		ToFVector(end),
		ToFQuat(rotation),
		ECollisionChannel::ECC_MAX,
		ToFCollisionShape(shape),
		CollisionParams, FCollisionResponseParams{});
	if (IsHit)
	{
		result->actor = (AActorOpaque*)Out.GetActor();
		result->distance = Out.Distance;
		result->location = ToVector3(Out.Location);
		result->normal = ToVector3(Out.Normal);
		result->impact_location = ToVector3(Out.ImpactPoint);
		result->impact_normal = ToVector3(Out.ImpactNormal);
		result->pentration_depth = Out.PenetrationDepth;
		result->start_penetrating = Out.bStartPenetrating;
	}

	return IsHit;
}

uint32_t SweepMulti(Vector3 start,
                    Vector3 end,
                    Quaternion rotation,
                    LineTraceParams params,
                    CollisionShape collision_shape,
                    uintptr_t max_results,
                    HitResult* results)
{
	TArray<FHitResult> Out;

	auto CollisionParams = FCollisionQueryParams();
	for (uintptr_t i = 0; i < params.ignored_actors_len; ++i)
	{
		CollisionParams.AddIgnoredActor((AActor*)params.ignored_actors[i]);
		CollisionParams.bFindInitialOverlaps = true;
		// TODO: Make configurable
		CollisionParams.bDebugQuery = true;
	}
	bool IsHit = GetModule().GameMode->GetWorld()->SweepMultiByChannel(
		Out,
		ToFVector(start),
		ToFVector(end),
		ToFQuat(rotation),
		ECollisionChannel::ECC_MAX,
		ToFCollisionShape(collision_shape),
		CollisionParams, FCollisionResponseParams{});

	uintptr_t Length = FGenericPlatformMath::Min(max_results, (uintptr_t)Out.Num());
	if (IsHit)
	{
		for (uintptr_t i = 0; i < Length; ++i)
		{
			FHitResult& Hit = Out[i];

			results[i].actor = (AActorOpaque*)Hit.GetActor();
			results[i].distance = Hit.Distance;
			results[i].location = ToVector3(Hit.Location);
			results[i].normal = ToVector3(Hit.Normal);
			results[i].impact_location = ToVector3(Hit.ImpactPoint);
			results[i].impact_normal = ToVector3(Hit.ImpactNormal);
			results[i].pentration_depth = Hit.PenetrationDepth;
			results[i].start_penetrating = Hit.bStartPenetrating;
		}
	}
	return Length;
}

void GetRegisteredClasses(UClassOpague** classes, uintptr_t* len)
{
	if (classes == nullptr)
	{
		*len = GetModule().GameMode->RegisteredClasses.Num();
		return;
	}
	auto GameMode = GetModule().GameMode;
	uintptr_t Count = *len;
	for (uintptr_t Idx = 0; Idx < Count; ++Idx)
	{
		classes[Idx] = (UClassOpague*)GameMode->RegisteredClasses[Idx].Get();
	}
}

UClassOpague* GetClass(const AActorOpaque* actor)
{
	return (UClassOpague*)ToAActor(actor)->GetClass();
}

uint32 IsMoveable(const AActorOpaque* actor)
{
	return ToAActor(actor)->IsRootComponentMovable();
}

void GetActorName(const AActorOpaque* actor, char* data, uintptr_t* len)
{
	// TODO: Implement
	//if(data == nullptr) {
	//    FString Name = ToAActor(actor)->GetActorNameOrLabel();
	//    auto Utf8 = TCHAR_TO_UTF8(*Name);
	//    auto Length = Utf8.Length();
	//}
}

void SetOwner(AActorOpaque* actor, const AActorOpaque* new_owner)
{
	ToAActor(actor)->SetOwner(ToAActor(new_owner));
}

uint32_t GetCollisionShape(const UPrimtiveOpaque* primitive, CollisionShape* out)
{
	const FCollisionShape UnrealShape = static_cast<const UPrimitiveComponent*>(primitive)->GetCollisionShape();


	if (UnrealShape.IsBox())
	{
		CollisionShape Shape;
		Shape.ty = CollisionShapeType::Box;
		const FVector HalfExtent = UnrealShape.GetBox();
		CollisionBox Box;
		Box.half_extent_x = HalfExtent.X;
		Box.half_extent_y = HalfExtent.Y;
		Box.half_extent_z = HalfExtent.Z;
		Shape.data.collision_box = Box;

		*out = Shape;
		return 1;
	}

	if (UnrealShape.IsSphere())
	{
		CollisionShape Shape;
		Shape.ty = CollisionShapeType::Sphere;
		const float Radius = UnrealShape.GetSphereRadius();
		CollisionSphere Sphere;

		Sphere.radius = Radius;
		Shape.data.sphere = Sphere;

		*out = Shape;
		return 1;
	}

	if (UnrealShape.IsCapsule())
	{
		CollisionShape Shape;
		Shape.ty = CollisionShapeType::Capsule;
		const float Radius = UnrealShape.GetCapsuleRadius();
		const float HalfHeight = UnrealShape.GetCapsuleHalfHeight();
		CollisionCapsule Capsule;
		Capsule.radius = Radius;
		Capsule.half_height = HalfHeight;
		Shape.data.capsule = Capsule;

		*out = Shape;
		return 1;
	}
	// TODO: Handle line instead?
	return 0;
}

void VisualLogLocation(Utf8Str category, const AActorOpaque* owner, Vector3 position, float radius, Color color)
{
	FString CategoryStr = ToFString(category);
	auto LogCat = FLogCategory<ELogVerbosity::Log, ELogVerbosity::All>(*CategoryStr);
	UE_VLOG_LOCATION(ToAActor(owner), LogCat, Log, ToFVector(position), radius, ToFColor(color), TEXT(""));
}

uint32_t GetEditorComponentUuids(const AActorOpaque* actor, Uuid* data, uintptr_t* len)
{
	ARustActor* Actor = Cast<ARustActor>(ToAActor(actor));
	if (Actor == nullptr)
	{
		return 0;
	}

	if (data == nullptr)
	{
		*len = Actor->EntityComponent->Components.Num();
		return 1;
	}
	uintptr_t Count = *len;
	uintptr_t Idx = 0;
	for (auto& Elem : Actor->EntityComponent->Components)
	{
		if (Idx == Count)
			return 1;
		data[Idx] = ToUuid(Elem.Key);
	}

	// We have only partially written to data
	*len = Idx;
	return 1;
}

uint32_t GetEditorComponentVector(const AActorOpaque* actor, Uuid uuid, Utf8Str field, Vector3* out)
{
	URustProperty* Prop = GetRustProperty(actor, uuid, field);
	if (Prop == nullptr)
		return 0;

	URustPropertyVector* VecProp = Cast<URustPropertyVector>(Prop);
	if (VecProp == nullptr)
		return 0;

	*out = ToVector3(VecProp->Data);
	return 1;
}

uint32_t GetEditorComponentFloat(const AActorOpaque* actor, Uuid uuid, Utf8Str field, float* out)
{
	URustProperty* Prop = GetRustProperty(actor, uuid, field);
	if (Prop == nullptr)
		return 0;

	URustPropertyFloat* FloatProp = Cast<URustPropertyFloat>(Prop);
	if (FloatProp == nullptr)
		return 0;

	*out = FloatProp->Data;
	return 1;
}

uint32_t GetEditorComponentBool(const AActorOpaque* actor, Uuid uuid, Utf8Str field, uint32_t* out)
{
	URustProperty* Prop = GetRustProperty(actor, uuid, field);
	if (Prop == nullptr)
		return 0;

	URustPropertyBool* BoolProp = Cast<URustPropertyBool>(Prop);
	if (BoolProp == nullptr)
		return 0;

	*out = BoolProp->Data == 1;
	return 1;
}

uint32_t GetEditorComponentQuat(const AActorOpaque* actor, Uuid uuid, Utf8Str field, Quaternion* out)
{
	URustProperty* Prop = GetRustProperty(actor, uuid, field);
	if (Prop == nullptr)
		return 0;

	URustPropertyQuaternion* QuatProp = Cast<URustPropertyQuaternion>(Prop);
	if (QuatProp == nullptr)
		return 0;

	*out = ToQuaternion(QuatProp->Data);
	return 1;
}
