use crate::Component;
use ffi::AActorOpaque;
use glam::{Quat, Vec3};
use unreal_ffi as ffi;

use crate::{
    core::{ActorPtr, Primitive, UnrealPtr},
    module::bindings,
};

#[derive(Debug)]
pub struct SweepResult {
    pub actor: Option<ActorPtr>,
    pub impact_location: Vec3,
    pub location: Vec3,
    pub penetration_depth: f32,
    pub normal: Vec3,
    pub impact_normal: Vec3,
    pub start_in_penentration: bool,
}

#[derive(Clone, Default)]
pub struct SweepParams {
    pub ignored_actors: Vec<*mut AActorOpaque>,
}

impl SweepParams {
    pub fn add_ignored_actor(mut self, actor: ActorPtr) -> Self {
        self.ignored_actors.push(actor.0);
        self
    }
}
pub fn sweep_multi(
    start: Vec3,
    end: Vec3,
    rotation: Quat,
    collision_shape: CollisionShape,
    max_results: usize,
    params: SweepParams,
) -> Option<Vec<SweepResult>> {
    let params = ffi::LineTraceParams {
        ignored_actors: params.ignored_actors.as_ptr(),
        ignored_actors_len: params.ignored_actors.len(),
    };
    let mut hits: Vec<ffi::HitResult> = Vec::new();
    hits.resize_with(max_results, Default::default);
    unsafe {
        let len = (bindings().physics_fns.sweep_multi)(
            start.into(),
            end.into(),
            rotation.into(),
            params,
            collision_shape.into(),
            max_results,
            hits.as_mut_ptr(),
        );
        hits.truncate(len as usize);
        if len > 0 {
            Some(
                hits.into_iter()
                    .map(|hit| {
                        let actor = if hit.actor.is_null() {
                            None
                        } else {
                            Some(ActorPtr(hit.actor))
                        };
                        SweepResult {
                            actor,
                            impact_location: hit.impact_location.into(),
                            location: hit.location.into(),
                            normal: hit.normal.into(),
                            penetration_depth: hit.pentration_depth,
                            start_in_penentration: hit.start_penetrating == 1,
                            impact_normal: hit.impact_normal.into(),
                        }
                    })
                    .collect(),
            )
        } else {
            None
        }
    }
}
#[derive(Copy, Clone)]
pub enum CollisionShape {
    Capsule { half_height: f32, radius: f32 },
    Box { half_extent: Vec3 },
    Sphere { radius: f32 },
}

impl CollisionShape {
    pub fn extent(self) -> Vec3 {
        match self {
            CollisionShape::Capsule {
                half_height,
                radius,
            } => Vec3::new(radius, radius, half_height),
            CollisionShape::Box { half_extent } => half_extent,
            CollisionShape::Sphere { radius } => Vec3::splat(radius),
        }
    }

    pub fn scale(self, amount: f32) -> Self {
        match self {
            CollisionShape::Capsule {
                half_height,
                radius,
            } => CollisionShape::Capsule {
                half_height: half_height * amount,
                radius: radius * amount,
            },
            CollisionShape::Box { half_extent } => CollisionShape::Box {
                half_extent: half_extent * amount,
            },
            CollisionShape::Sphere { radius } => CollisionShape::Sphere {
                radius: radius * amount,
            },
        }
    }

    pub fn inflate(self, amount: f32) -> Self {
        match self {
            CollisionShape::Capsule {
                half_height,
                radius,
            } => CollisionShape::Capsule {
                half_height: half_height + amount,
                radius: radius + amount,
            },
            CollisionShape::Box { half_extent } => CollisionShape::Box {
                half_extent: half_extent + amount,
            },
            CollisionShape::Sphere { radius } => CollisionShape::Sphere {
                radius: radius + amount,
            },
        }
    }
}

impl From<CollisionShape> for ffi::CollisionShape {
    fn from(val: CollisionShape) -> Self {
        match val {
            CollisionShape::Box { half_extent } => ffi::CollisionShape {
                ty: ffi::CollisionShapeType::Box,
                data: ffi::CollisionShapeUnion {
                    collision_box: ffi::CollisionBox {
                        half_extent_x: half_extent.x,
                        half_extent_y: half_extent.y,
                        half_extent_z: half_extent.z,
                    },
                },
            },
            CollisionShape::Capsule {
                half_height,
                radius,
            } => ffi::CollisionShape {
                data: ffi::CollisionShapeUnion {
                    capsule: ffi::CollisionCapsule {
                        radius,
                        half_height,
                    },
                },
                ty: ffi::CollisionShapeType::Capsule,
            },
            CollisionShape::Sphere { radius } => ffi::CollisionShape {
                data: ffi::CollisionShapeUnion {
                    sphere: ffi::CollisionSphere { radius },
                },
                ty: ffi::CollisionShapeType::Sphere,
            },
        }
    }
}

#[derive(Default, Component)]
#[uuid = "ffc10b5c-635c-43ce-8288-e3c6f6d67e36"]
pub struct PhysicsComponent {
    #[reflect(skip)]
    pub ptr: UnrealPtr<Primitive>,
    pub is_simulating: bool,
    pub velocity: Vec3,
}

impl PhysicsComponent {
    pub fn new(ptr: UnrealPtr<Primitive>) -> Self {
        let mut p = Self {
            ptr,
            ..Default::default()
        };
        p.download_state();
        p
    }

    pub fn get_collision_shape(&self) -> CollisionShape {
        unsafe {
            let mut shape = ffi::CollisionShape::default();
            assert!(
                (bindings().physics_fns.get_collision_shape)(self.ptr.ptr, &mut shape) == 1
            );
            match shape.ty {
                ffi::CollisionShapeType::Capsule => CollisionShape::Capsule {
                    half_height: shape.data.capsule.half_height,
                    radius: shape.data.capsule.radius,
                },
                ffi::CollisionShapeType::Box => CollisionShape::Box {
                    half_extent: Vec3::new(
                        shape.data.collision_box.half_extent_x,
                        shape.data.collision_box.half_extent_y,
                        shape.data.collision_box.half_extent_y,
                    ),
                },
                ffi::CollisionShapeType::Sphere => CollisionShape::Sphere {
                    radius: shape.data.sphere.radius,
                },
            }
        }
    }
    pub fn download_state(&mut self) {
        unsafe {
            self.is_simulating = (bindings().physics_fns.is_simulating)(self.ptr.ptr) == 1;
            self.velocity = (bindings().physics_fns.get_velocity)(self.ptr.ptr).into();
        }
    }

    pub fn upload_state(&mut self) {
        unsafe {
            (bindings().physics_fns.set_velocity)(self.ptr.ptr, self.velocity.into());
        }
    }

    pub fn add_impulse(&mut self, impulse: Vec3) {
        unsafe {
            (bindings().physics_fns.add_impulse)(self.ptr.ptr, impulse.into());
        }
    }

    pub fn add_force(&mut self, force: Vec3) {
        unsafe {
            (bindings().physics_fns.add_force)(self.ptr.ptr, force.into());
        }
    }
}
