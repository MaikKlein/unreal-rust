use std::collections::HashMap;

use crate::ffi;
use glam::{Quat, Vec3};

use crate::core::ActorPtr;
use crate::ecs::entity::Entity;
use crate::module::bindings;
use crate::physics::CollisionShape;

#[derive(Default)]
pub struct UnrealApi {
    // TODO: Implement unregister.
    pub actor_to_entity: HashMap<ActorPtr, Entity>,
    pub entity_to_actor: HashMap<Entity, ActorPtr>,
}

#[derive(Default)]
pub struct SweepParams {
    pub ignored_entities: Vec<Entity>,
}

impl SweepParams {
    pub fn add_ignored_entity(mut self, entity: Entity) -> Self {
        self.ignored_entities.push(entity);
        self
    }
}

#[derive(Debug)]
pub struct SweepHit {
    /// The entity that was hit
    pub entity: Entity,
    /// Location in world space of the actual contact of the trace shape (box, sphere, ray, etc) with the impacted object.
    pub impact_location: Vec3,
    /// Normal of the hit in world space, for the object that was hit by the sweep, if any
    pub impact_normal: Vec3,
    /// If this test started in penetration (bStartPenetrating is true) and a depenetration vector can be computed, this value is the distance along Normal that will result in moving out of penetration.
    pub penetration_depth: f32,
    /// The location in world space where the moving shape would end up against the impacted object, if there is a hit.
    pub location: Vec3,
    /// Normal of the hit in world space, for the object that was swept.
    pub normal: Vec3,
    pub start_in_penentration: bool,
}

#[derive(Default)]
pub struct LineTraceParams {
    pub ignored_entities: Vec<Entity>,
}

impl LineTraceParams {
    pub fn add_ignored_entity(mut self, entity: Entity) -> Self {
        self.ignored_entities.push(entity);
        self
    }
}

#[derive(Debug)]
pub struct LineTraceHit {
    /// The entity that was hit
    pub entity: Entity,
    pub location: Vec3,
    pub normal: Vec3,
}

impl UnrealApi {
    pub fn register_actor(&mut self, actor: ActorPtr, entity: Entity) {
        self.actor_to_entity.insert(actor, entity);
        self.entity_to_actor.insert(entity, actor);
    }
    pub fn sweep(
        &self,
        start: Vec3,
        end: Vec3,
        rotation: Quat,
        collision_shape: CollisionShape,
        params: SweepParams,
    ) -> Option<SweepHit> {
        let ignored_actors: Vec<_> = params
            .ignored_entities
            .iter()
            .filter_map(|entity| self.entity_to_actor.get(entity))
            .map(|actor| actor.0)
            .collect();
        let params = ffi::LineTraceParams {
            ignored_actors: ignored_actors.as_ptr(),
            ignored_actors_len: ignored_actors.len(),
        };
        let mut hit = ffi::HitResult::default();
        unsafe {
            if (bindings().physics_fns.sweep)(
                start.into(),
                end.into(),
                rotation.into(),
                params,
                collision_shape.into(),
                &mut hit,
            ) == 1
            {
                let entity = self
                    .actor_to_entity
                    .get(&ActorPtr(hit.actor))
                    .copied()
                    .expect("We hit an unknown actor. Please create an issue.");

                Some(SweepHit {
                    entity,
                    impact_location: hit.impact_location.into(),
                    location: hit.location.into(),
                    normal: hit.normal.into(),
                    penetration_depth: hit.pentration_depth,
                    start_in_penentration: hit.start_penetrating == 1,
                    impact_normal: hit.impact_normal.into(),
                })
            } else {
                None
            }
        }
    }

    pub fn line_trace(
        &self,
        start: Vec3,
        end: Vec3,
        params: LineTraceParams,
    ) -> Option<LineTraceHit> {
        let ignored_actors: Vec<_> = params
            .ignored_entities
            .iter()
            .filter_map(|entity| self.entity_to_actor.get(entity))
            .map(|actor| actor.0)
            .collect();
        let params = ffi::LineTraceParams {
            ignored_actors: ignored_actors.as_ptr(),
            ignored_actors_len: ignored_actors.len(),
        };
        let mut hit = ffi::HitResult::default();
        unsafe {
            if (bindings().physics_fns.line_trace)(start.into(), end.into(), params, &mut hit)
                == 1
            {
                let entity = self
                    .actor_to_entity
                    .get(&ActorPtr(hit.actor))
                    .copied()
                    .expect("We hit an unknown actor. Please create an issue.");
                Some(LineTraceHit {
                    entity,
                    location: hit.location.into(),
                    normal: hit.normal.into(),
                })
            } else {
                None
            }
        }
    }
}
