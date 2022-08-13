#include "RustUtils.h"
#include "Modules/ModuleManager.h"
#include "RustPlugin.h"
#include "EngineUtils.h"
#include "RustActor.h"

UnrealBindings CreateBindings()
{
	
	EditorComponentFns editor_component_fns;
	editor_component_fns.get_editor_component_bool = &GetEditorComponentBool;
	editor_component_fns.get_editor_component_float = &GetEditorComponentFloat;
	editor_component_fns.get_editor_component_quat = &GetEditorComponentQuat;
	editor_component_fns.get_editor_component_vector = &GetEditorComponentVector;
	editor_component_fns.get_editor_component_uobject = &GetEditorComponentUObject;
	editor_component_fns.get_editor_components = &GetEditorComponentUuids;
	
	UnrealPhysicsBindings physics_bindings = {};
	physics_bindings.add_force = &AddForce;
	physics_bindings.add_impulse = &AddImpulse;
	physics_bindings.set_velocity = &SetVelocity;
	physics_bindings.get_velocity = &GetVelocity;
	physics_bindings.is_simulating = &IsSimulating;
	physics_bindings.line_trace = &LineTrace;
	physics_bindings.get_bounding_box_extent = &GetBoundingBoxExtent;
	physics_bindings.sweep = &Sweep;
	physics_bindings.sweep_multi = &SweepMulti;
	physics_bindings.get_collision_shape = &GetCollisionShape;

	UnrealBindings b = {};
	b.physics_bindings = physics_bindings;
	b.editor_component_fns = editor_component_fns;
	b.get_spatial_data = &GetSpatialData;
	b.set_spatial_data = &SetSpatialData;
	b.log = &Log;
	b.iterate_actors = &IterateActors;
	b.get_action_state = &GetActionState;
	b.get_axis_value = &GetAxisValue;
	b.set_entity_for_actor = &SetEntityForActor;
	b.spawn_actor = &SpawnActor;
	b.set_view_target = &SetViewTarget;
	b.get_mouse_delta = &GetMouseDelta;
	b.get_actor_components = &GetActorComponents;
	b.visual_log_segment = &VisualLogSegment;
	b.visual_log_capsule = &VisualLogCapsule;
	b.visual_log_location = &VisualLogLocation;
	b.get_root_component = &GetRootComponent;
	b.get_registered_classes = &GetRegisteredClasses;
	b.get_class = &GetClass;
	b.is_moveable = &IsMoveable;
	b.set_owner = &SetOwner;
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

FRustPluginModule& GetModule()
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
	return FString(Str.len, UTF8_TO_TCHAR(Str.ptr));
}

FRustProperty2* GetRustProperty(const AActorOpaque* actor, Uuid uuid, Utf8Str field)
{
	UEntityComponent* EntityComponent = ToAActor(actor)->FindComponentByClass<UEntityComponent>();
	if(EntityComponent == nullptr)
	{
		return nullptr;
	}
	FString FieldName = ToFString(field);

	FDynamicRustComponent2* Comp = EntityComponent->Components.Find(ToFGuid(uuid).ToString());
	if(Comp == nullptr)
		return nullptr;
	return Comp->Fields.Find(FieldName);
}
