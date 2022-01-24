use bevy_ecs::prelude::*;
use unreal_api::{
    core::{
        ActorComponent, ActorPtr, CameraComponent, CoreStage, Frame, MovementComponent,
        ParentComponent, PlayerInputComponent, SpatialComponent,
    },
    ffi,
    math::{Quat, Vec3},
    module::{bindings, InitUserModule, UserModule},
};
use unreal_reflect::registry::ReflectionRegistry;

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

fn debug_velocity(mut query: Query<(&SpatialComponent, &ActorComponent, &MovementComponent)>) {
    for (spatial, actor, movement) in query.iter() {
        let end = spatial.position + movement.velocity;
        (bindings().visual_log_segment)(
            actor.ptr.0,
            spatial.position.into(),
            end.into(),
            ffi::Color::RED,
        );
    }
}
fn compute_movement_velocity(mut query: Query<(&PlayerInputComponent, &mut MovementComponent)>) {
    for (input, mut movement) in query.iter_mut() {
        let mut dir = movement.view * input.direction;
        dir.z = 0.0;

        movement.velocity = dir.normalize_or_zero() * 500.0 * 1.0;
    }
}

fn perform_movement(
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

fn update_parent_view(
    mut movement: Query<&mut MovementComponent>,
    camera: Query<(&ParentComponent, &SpatialComponent, &CameraComponent)>,
) {
    for (parent, spatial, _) in camera.iter() {
        if let Some(mut movement) = movement
            .get_component_mut::<MovementComponent>(parent.parent)
            .ok()
        {
            movement.view = spatial.rotation;
        }
    }
}

fn rotate_camera(mut query: Query<(&mut SpatialComponent, &mut CameraComponent)>) {
    fn lerp(start: f32, end: f32, t: f32) -> f32 {
        start * (1.0 - t) + end * t
    }
    let mut x = 0.0;
    let mut y = 0.0;

    let max_angle = 85.0f32.to_radians();
    (bindings().get_mouse_delta)(&mut x, &mut y);
    for (mut spatial, mut cam) in query.iter_mut() {
        let speed = 0.05;
        cam.x += x * speed;
        cam.y = f32::clamp(cam.y + y * speed, -max_angle, max_angle);

        let smooth = 0.8;
        cam.current_x = lerp(cam.current_x, cam.x, smooth);
        cam.current_y = lerp(cam.current_y, cam.y, smooth);

        spatial.rotation =
            Quat::from_rotation_z(cam.current_x) * Quat::from_rotation_y(-cam.current_y);
    }
}

fn spawn_camera(
    mut commands: Commands,
    mut query: Query<(Entity, &ActorComponent, Added<PlayerInputComponent>)>,
) {
    for (entity, _, added) in query.iter_mut() {
        if !added {
            continue;
        }
        let pos = Vec3::new(-2587.0, -1800.0, 150.0);
        let actor = (bindings().spawn_actor)(
            ffi::ActorClass::CameraActor,
            pos.into(),
            Quat::from_rotation_x(0.0).into(),
            Vec3::ONE.into(),
        );
        (bindings().set_view_target)(actor);

        commands.spawn().insert_bundle((
            SpatialComponent {
                position: pos,
                ..Default::default()
            },
            ActorComponent {
                ptr: ActorPtr(actor),
            },
            CameraComponent::default(),
            ParentComponent { parent: entity },
        ));
    }
}

fn update_camera(
    mut query: Query<(Entity, &ParentComponent, &CameraComponent)>,
    mut spatial_query: Query<&mut SpatialComponent>,
) {
    for (entity, parent, _) in query.iter_mut() {
        let spatial_parent = spatial_query
            .get_component::<SpatialComponent>(parent.parent)
            .ok()
            .cloned();
        let spatial = spatial_query
            .get_component_mut::<SpatialComponent>(entity)
            .ok();
        if let (Some(mut spatial), Some(parent)) = (spatial, spatial_parent) {
            let local_offset = spatial.rotation * Vec3::new(-500.0, 0.0, 150.0);
            spatial.position = parent.position + local_offset;
        }
    }
}

pub struct MyModule;

impl InitUserModule for MyModule {
    fn initialize() -> Self {
        Self {}
    }
}
impl UserModule for MyModule {
    fn register(&self, _registry: &mut ReflectionRegistry) {}

    fn systems(&self, _startup: &mut Schedule, update: &mut Schedule) {
        update.add_system_to_stage(CoreStage::Update, spawn_camera.system());
        update.add_system_to_stage(CoreStage::Update, update_player_input.system());
        update.add_system_to_stage(CoreStage::Update, compute_movement_velocity.system());
        update.add_system_to_stage(CoreStage::Update, perform_movement.system());
        update.add_system_to_stage(CoreStage::Update, debug_velocity.system());
        update.add_system_to_stage(CoreStage::Update, rotate_camera.system());
        update.add_system_to_stage(CoreStage::Update, update_parent_view.system());
        update.add_system_to_stage(CoreStage::Update, update_camera.system());
    }
}
unreal_api::implement_unreal_module!(MyModule);
