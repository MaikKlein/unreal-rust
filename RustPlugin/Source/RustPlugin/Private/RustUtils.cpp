#include "RustUtils.h"
#include "Modules/ModuleManager.h"
#include "RustPlugin.h"
#include "EngineUtils.h"
#include "RustActor.h"

UnrealBindings CreateBindings()
{
	SoundFns sound_fns;
	sound_fns.play_sound_at_location = PlaySoundAtLocation;
	
	EditorComponentFns editor_component_fns;
	editor_component_fns.get_editor_component_bool = &GetEditorComponentBool;
	editor_component_fns.get_editor_component_float = &GetEditorComponentFloat;
	editor_component_fns.get_editor_component_quat = &GetEditorComponentQuat;
	editor_component_fns.get_editor_component_vector = &GetEditorComponentVector;
	editor_component_fns.get_editor_component_uobject = &GetEditorComponentUObject;
	editor_component_fns.get_editor_components = &GetEditorComponentUuids;

	PhysicsFns physics_fns = {};
	 physics_fns.add_force = &AddForce;
	 physics_fns.add_impulse = &AddImpulse;
	 physics_fns.set_velocity = &SetVelocity;
	 physics_fns.get_velocity = &GetVelocity;
	 physics_fns.is_simulating = &IsSimulating;
	 physics_fns.line_trace = &LineTrace;
	 physics_fns.get_bounding_box_extent = &GetBoundingBoxExtent;
	 physics_fns.sweep = &Sweep;
	 physics_fns.sweep_multi = &SweepMulti;
	 physics_fns.get_collision_shape = &GetCollisionShape;

	ActorFns actor_fns = {};
	actor_fns.get_spatial_data = &GetSpatialData;
	actor_fns.set_spatial_data = &SetSpatialData;
	actor_fns.set_entity_for_actor = &SetEntityForActor;
	actor_fns.set_view_target = &SetViewTarget;
	actor_fns.get_actor_components = &GetActorComponents;
	actor_fns.get_registered_classes = &GetRegisteredClasses;
	actor_fns.get_class = &GetClass;
	actor_fns.set_owner = &SetOwner;
	actor_fns.get_actor_name = &GetActorName;
	actor_fns.is_moveable = &IsMoveable;
	actor_fns.register_actor_on_overlap = &RegisterActorOnOverlap;
	actor_fns.get_root_component = &GetRootComponent;
	
	UnrealBindings b = {};
	b.actor_fns = actor_fns;
	b.sound_fns = sound_fns;
	b.physics_fns = physics_fns;
	b.editor_component_fns = editor_component_fns;
	b.log = &Log;
	b.iterate_actors = &IterateActors;
	b.get_action_state = &GetActionState;
	b.get_axis_value = &GetAxisValue;
	b.spawn_actor = &SpawnActor;
	b.get_mouse_delta = &GetMouseDelta;
	b.visual_log_segment = &VisualLogSegment;
	b.visual_log_capsule = &VisualLogCapsule;
	b.visual_log_location = &VisualLogLocation;
	return b;
}

Quaternion ToQuaternion(FQuat q)
{
	Quaternion r;
	r.x = q.X;
	r.y = q.Y;
	r.z = q.Z;
	r.w = q.W;
	return r;
}

Vector3 ToVector3(FVector v)
{
	Vector3 r;
	r.x = v.X;
	r.y = v.Y;
	r.z = v.Z;
	return r;
}

FVector ToFVector(Vector3 v)
{
	return FVector(v.x, v.y, v.z);
}

FQuat ToFQuat(Quaternion q)
{
	return FQuat(q.x, q.y, q.z, q.w);
}

AActor* ToAActor(const AActorOpaque* actor)
{
	return (AActor*)actor;
}

AActor* ToAActor(AActorOpaque* actor)
{
	return (AActor*)actor;
}

FGuid ToFGuid(Uuid uuid)
{
	return FGuid(uuid.a, uuid.b, uuid.c, uuid.d);
}

Uuid ToUuid(FGuid guid)
{
	Uuid uuid;
	uuid.a = guid.A;
	uuid.b = guid.B;
	uuid.c = guid.C;
	uuid.d = guid.D;
	return uuid;
}

FRustPluginModule& GetRustModule()
{
	return FModuleManager::LoadModuleChecked<FRustPluginModule>(TEXT("RustPlugin"));
}

FColor ToFColor(Color c)
{
	return FColor(c.r, c.g, c.b, c.a);
}

FCollisionShape ToFCollisionShape(CollisionShape Shape)
{
	if (Shape.ty == CollisionShapeType::Box)
	{
		return FCollisionShape::MakeBox(FVector3f(
			Shape.data.collision_box.half_extent_x,
			Shape.data.collision_box.half_extent_y,
			Shape.data.collision_box.half_extent_z
		));
	}
	if (Shape.ty == CollisionShapeType::Sphere)
	{
		return FCollisionShape::MakeSphere(Shape.data.sphere.radius);
	}

	if (Shape.ty == CollisionShapeType::Capsule)
	{
		return FCollisionShape::MakeCapsule(
			Shape.data.capsule.radius,
			Shape.data.capsule.half_height
		);
	}

	// TODO: Unreal way?
	abort();
}

FString ToFString(Utf8Str Str)
{
	if(Str.len == 0)
		return FString();
	
	return FString(Str.len, UTF8_TO_TCHAR(Str.ptr));
}

FRustProperty* GetRustProperty(const AActorOpaque* actor, Uuid uuid, Utf8Str field)
{
	AActor* Actor = ToAActor(actor);
	if (Actor == nullptr)
		return nullptr;

	UEntityComponent* EntityComponent = Actor->FindComponentByClass<UEntityComponent>();
	if (EntityComponent == nullptr)
		return nullptr;
	
	FString FieldName = ToFString(field);

	FDynamicRustComponent* Comp = EntityComponent->Components.Find(ToFGuid(uuid).ToString());
	if (Comp == nullptr)
		return nullptr;
	return Comp->Fields.Find(FieldName);
}
