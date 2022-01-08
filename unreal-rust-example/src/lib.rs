use std::{collections::HashMap, ffi::CStr};

use bevy_ecs::prelude::*;
use unreal_api::{
    ffi::{self, AActorOpaque, ActionState},
    iterate_actors,
    math::{Quat, Vec3},
    module::{bindings, UnrealModule}, core::{ActorComponent, SpatialComponent, Frame, CoreStage, PlayerInputComponent, MovementComponent},
};
use unreal_reflect::{impl_component, registry::ReflectionRegistry, TypeUuid};

fn update_player_input(mut query: Query<(&ActorComponent, &mut PlayerInputComponent)>) {
    for (actor, mut input) in query.iter_mut() {
        let forward = {
            let s = "MoveForward";
            let mut value = 0.0;
            (bindings().get_axis_value)(s.as_ptr() as *const _, s.len(), &mut value);
            value
        };
        let right = {
            let s = "MoveRight";
            let mut value = 0.0;
            (bindings().get_axis_value)(s.as_ptr() as *const _, s.len(), &mut value);
            value
        };

        input.direction = Vec3::new(forward, right, 0.0).normalize_or_zero();
    }
}

fn compute_movement_velocity(mut query: Query<(&PlayerInputComponent, &mut MovementComponent)>) {
    for (input, mut movement) in query.iter_mut() {
        movement.velocity = input.direction * 500.0;
    }
}

fn update_velocity(
    frame: Res<Frame>,
    mut query: Query<(&mut SpatialComponent, &MovementComponent)>,
) {
    for (mut spatial, movement) in query.iter_mut() {
        spatial.position += movement.velocity * frame.dt;

        if movement.velocity.length() > 0.2 {
            let velocity_dir = movement.velocity.normalize_or_zero();
            let target_rot = Quat::from_rotation_z(f32::atan2(velocity_dir.y, velocity_dir.x));
            spatial.rotation = Quat::lerp(spatial.rotation, target_rot, frame.dt * 10.0);
        }
    }
}
fn rotate(frame: Res<Frame>, mut query: Query<&mut SpatialComponent>) {
    for mut spatial in query.iter_mut() {
        spatial.rotation = Quat::from_rotation_z(1.5 * frame.dt) * spatial.rotation;
    }
}

pub struct MyModule;

impl UnrealModule for MyModule {
    fn initialize() -> Self {
        Self {}
    }

    fn register(registry:&mut ReflectionRegistry) {

    }

    fn systems(_startup: &mut Schedule, update: &mut Schedule) {
        update.add_system_to_stage(CoreStage::Update, update_player_input.system());
        update.add_system_to_stage(CoreStage::Update, compute_movement_velocity.system());
        update.add_system_to_stage(CoreStage::Update, update_velocity.system());
    }
}
unreal_api::implement_unreal_module!(MyModule);
