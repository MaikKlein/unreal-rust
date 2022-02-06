use bevy_ecs::prelude::*;
use unreal_api::{
    core::{
        ActorComponent, ActorPtr, CameraComponent, CoreStage, Frame, MovementComponent,
        ParentComponent, PhysicsComponent, PlayerInputComponent, TransformComponent,
    },
    ffi::{self, HitResult},
    input::Input,
    math::{Quat, Vec3},
    module::{bindings, InitUserModule, UserModule},
    physics::{self, sweep, SweepParams},
};
use unreal_reflect::{impl_component, registry::ReflectionRegistry, TypeUuid};

fn project_onto_plane(dir: Vec3, normal: Vec3) -> Vec3 {
    dir - normal * Vec3::dot(dir, normal)
}

#[derive(Debug, Copy, Clone)]
pub enum MovementState {
    Walking,
    Falling,
}
impl Default for MovementState {
    fn default() -> Self {
        Self::Walking
    }
}

#[derive(Default, Debug, TypeUuid)]
#[uuid = "ac41cdd4-3311-45ef-815c-9a31adbe4098"]
pub struct CharacterControllerComponent {
    pub velocity: Vec3,
    pub camera_view: Quat,
    pub movement_state: MovementState,
    pub visual_rotation: Quat,
}

impl_component!(CharacterControllerComponent);
#[derive(Default, Debug, TypeUuid)]
#[uuid = "16ca6de6-7a30-412d-8bef-4ee96e18a101"]
pub struct CharacterConfigComponent {
    pub max_movement_speed: f32,
    pub gravity_dir: Vec3,
    pub gravity_strength: f32,
    pub max_walkable_slope: f32,
}
impl_component!(CharacterConfigComponent);

pub struct PlayerInput;
impl PlayerInput {
    pub const MOVE_FORWARD: &'static str = "MoveForward";
    pub const MOVE_RIGHT: &'static str = "MoveRight";
}
fn register_player_input(mut input: ResMut<Input>) {
    input.register_axis_binding(PlayerInput::MOVE_FORWARD);
    input.register_axis_binding(PlayerInput::MOVE_RIGHT);
}

fn update_player_input(
    input: Res<Input>,
    mut query: Query<(&ActorComponent, &mut PlayerInputComponent)>,
) {
    for (actor, mut player_input) in query.iter_mut() {
        let forward = input
            .get_axis_value(PlayerInput::MOVE_FORWARD)
            .unwrap_or(0.0);
        let right = input.get_axis_value(PlayerInput::MOVE_RIGHT).unwrap_or(0.0);
        player_input.direction = Vec3::new(forward, right, 0.0).normalize_or_zero();
    }
}

pub enum MovementHit {
    Slope { normal: Vec3 },
    Wall { normal: Vec3 },
    Landing,
    Falling,
}

pub fn movement_hit(
    actor: &ActorComponent,
    transform: &TransformComponent,
    physics: &PhysicsComponent,
    config: &CharacterConfigComponent,
    controller: &CharacterControllerComponent,
    dt: f32,
) -> Option<MovementHit> {
    let mut params = SweepParams::default();

    params.ignored_actors.push(actor.ptr.0);
    if let Some(hit) = sweep(
        transform.position,
        transform.position + controller.velocity * dt,
        transform.rotation,
        physics,
        params,
    ) {
        let is_moving_against_wall = Vec3::dot(hit.normal, controller.velocity) < 0.0;
        if f32::abs(Vec3::dot(hit.normal, Vec3::Z)) < 0.2 && is_moving_against_wall {
            let wall_normal = Vec3::new(hit.normal.x, hit.normal.y, 0.0);
            return Some(MovementHit::Wall {
                normal: wall_normal,
            });
        }
        if Vec3::dot(hit.normal, Vec3::Z) > 0.5 && is_moving_against_wall {
            return Some(MovementHit::Slope { normal: hit.normal });
        }
    }
    let mut params = SweepParams::default();

    params.ignored_actors.push(actor.ptr.0);
    if let Some(hit) = sweep(
        transform.position,
        transform.position + Vec3::new(0.0, 0.0, -100.0),
        transform.rotation,
        physics,
        params,
    ) {
        if Vec3::distance(hit.location, transform.position) > 5.0 {
            return Some(MovementHit::Falling);
        }

        if matches!(controller.movement_state, MovementState::Falling) {
            return Some(MovementHit::Landing);
        }
    }

    None
}

fn character_control_system(
    input: Res<Input>,
    frame: Res<Frame>,
    mut query: Query<(
        &mut TransformComponent,
        &mut CharacterControllerComponent,
        &CharacterConfigComponent,
        &PhysicsComponent,
        &ActorComponent,
    )>,
) {
    let forward = input
        .get_axis_value(PlayerInput::MOVE_FORWARD)
        .unwrap_or(0.0);
    let right = input.get_axis_value(PlayerInput::MOVE_RIGHT).unwrap_or(0.0);
    let player_input = Vec3::new(forward, right, 0.0).normalize_or_zero();


    for (mut transform, mut controller, config, physics, actor) in query.iter_mut() {
        controller.velocity = player_input * config.max_movement_speed;

        if let Some(hit) = movement_hit(actor, &transform, physics, config, &controller, frame.dt) {
            match hit {
                MovementHit::Falling => controller.movement_state = MovementState::Falling,
                MovementHit::Landing => {
                    controller.movement_state = MovementState::Walking;
                    controller.velocity.z = 0.0;
                }
                MovementHit::Slope { normal } => {
                    controller.velocity = project_onto_plane(controller.velocity, normal)
                        .normalize_or_zero()
                        * config.max_movement_speed
                }
                MovementHit::Wall { normal } => {
                    controller.velocity = project_onto_plane(controller.velocity, normal)
                }
            }
        }


    }
}

fn debug_velocity(
    query: Query<(
        &TransformComponent,
        &ActorComponent,
        &MovementComponent,
        &PhysicsComponent,
    )>,
) {
    //for (spatial, actor, movement, physics) in query.iter() {
    //    let end = spatial.position + movement.velocity;
    //    let start = spatial.position + spatial.forward();
    //    let end = spatial.position + spatial.forward() * 50.0;

    //    let ignored_actors = vec![actor.ptr.0];
    //    let params = ffi::LineTraceParams {
    //        ignored_actors: ignored_actors.as_ptr(),
    //        ignored_actors_len: ignored_actors.len(),
    //    };
    //    let mut hit = HitResult::default();
    //    if (bindings().physics_bindings.sweep)(
    //        start.into(),
    //        end.into(),
    //        spatial.rotation.into(),
    //        params,
    //        physics.ptr.ptr,
    //        &mut hit,
    //    ) == 1
    //    {
    //        (bindings().visual_log_segment)(
    //            actor.ptr.0,
    //            spatial.position.into(),
    //            hit.location,
    //            ffi::Color::RED,
    //        );
    //        (bindings().visual_log_segment)(
    //            actor.ptr.0,
    //            spatial.position.into(),
    //            hit.impact_location,
    //            ffi::Color::GREEN,
    //        );

    //        log::info!("depth: {}", hit.pentration_depth);
    //    }
    //}
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
    mut query: Query<(
        &mut TransformComponent,
        &mut MovementComponent,
        &PhysicsComponent,
        &ActorComponent,
    )>,
) {
    for (mut spatial, mut movement, physics, actor) in query.iter_mut() {
        {
            let mut params = SweepParams::default();

            params.ignored_actors.push(actor.ptr.0);
            let is_falling = if let Some(hit) = sweep(
                spatial.position,
                spatial.position + Vec3::new(0.0, 0.0, -100.0),
                spatial.rotation,
                physics,
                params,
            ) {
                Vec3::distance(hit.location, spatial.position) > 5.0
            } else {
                true
            };
            movement.is_falling = is_falling;
        }
        if movement.is_falling {
            movement.velocity += Vec3::new(0.0, 0.0, -981.0);
        }
        let mut params = SweepParams::default();

        params.ignored_actors.push(actor.ptr.0);
        if let Some(hit) = sweep(
            spatial.position,
            spatial.position + movement.velocity * frame.dt,
            spatial.rotation,
            physics,
            params,
        ) {
            let is_moving_against_wall = Vec3::dot(hit.normal, movement.velocity) < 0.0;
            if f32::abs(Vec3::dot(hit.normal, Vec3::Z)) < 0.2 && is_moving_against_wall {
                let wall_normal = Vec3::new(hit.normal.x, hit.normal.y, 0.0);
                movement.velocity = project_onto_plane(movement.velocity, wall_normal);
            } else if Vec3::dot(hit.normal, Vec3::Z) > 0.5 && is_moving_against_wall {
                movement.velocity =
                    project_onto_plane(movement.velocity, hit.normal).normalize_or_zero() * 500.0;
            } else if Vec3::dot(hit.normal, Vec3::Z) > 0.9 && movement.is_falling {
                movement.velocity.z = 0.0;
            }
            //let offset = Vec3::Z * -110.0;
            //unreal_api::log::visual_log_capsule(
            //    actor.ptr,
            //    hit.location,
            //    spatial.rotation,
            //    110.0,
            //    42.0,
            //    ffi::Color::GREEN,
            //);
            ////let normal = Vec3::new(hit.normal.x, 0.0, hit.normal.z).normalize();
            //if hit.penetration_depth > 0.0 {
            //    log::info!("{}", hit.penetration_depth);
            //    let new_pos = (spatial.position + movement.velocity * frame.dt)
            //        + hit.normal * hit.penetration_depth;
            //    movement.velocity = (spatial.position - new_pos);
            //}
            //movement.velocity = Vec3::ZERO;
            //if normal.dot(Vec3::Z) > 0.3 {
            //    let fwd = Vec3::cross(-normal, spatial.right());
            //    movement.velocity = fwd * 500.0;
            //}
            //else {
            //    movement.velocity = Vec3::ZERO;
            //}
        } else {
        }
        //unreal_api::log::visual_log_capsule(
        //    actor.ptr,
        //    spatial.position + movement.velocity * frame.dt,
        //    spatial.rotation,
        //    110.0,
        //    42.0,
        //    ffi::Color::RED,
        //);
        //(bindings().visual_log_segment)(
        //    actor.ptr.0,
        //    spatial.position.into(),
        //    (spatial.position + movement.velocity).into(),
        //    ffi::Color::GREEN,
        //);
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
    camera: Query<(&ParentComponent, &TransformComponent, &CameraComponent)>,
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

fn rotate_camera(mut query: Query<(&mut TransformComponent, &mut CameraComponent)>) {
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
            TransformComponent {
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
    mut spatial_query: Query<&mut TransformComponent>,
) {
    for (entity, parent, _) in query.iter_mut() {
        let spatial_parent = spatial_query
            .get_component::<TransformComponent>(parent.parent)
            .ok()
            .cloned();
        let spatial = spatial_query
            .get_component_mut::<TransformComponent>(entity)
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

    fn systems(&self, startup: &mut Schedule, update: &mut Schedule) {
        startup.add_system_to_stage(CoreStage::Startup, register_player_input.system());
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
