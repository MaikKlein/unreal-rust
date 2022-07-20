use std::collections::HashMap;

use bevy_ecs::prelude::*;
use unreal_api::{
    core::{
        ActorComponent, ActorPtr, CoreStage, Frame, ParentComponent, PhysicsComponent,
        TransformComponent,
    },
    ffi::{self, UClassOpague},
    input::Input,
    log::LogCategory,
    math::{Quat, Vec3, Vec3Swizzles},
    module::{bindings, InitUserModule, Module, UserModule},
    physics::{sweep, SweepParams},
    register_components2,
};
use unreal_reflect::Component;

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum Class {
    Player = 0,
}

impl Class {
    pub fn from(i: u32) -> Option<Self> {
        match i {
            0 => Some(Self::Player),
            _ => None,
        }
    }
}

#[derive(Default)]
pub struct ClassesResource {
    classes: HashMap<*mut ffi::UClassOpague, Class>,
}
unsafe impl Send for ClassesResource {}
unsafe impl Sync for ClassesResource {}

fn project_onto_plane(dir: Vec3, normal: Vec3) -> Vec3 {
    dir - normal * Vec3::dot(dir, normal)
}

#[derive(Debug, Copy, Clone)]
pub enum MovementState {
    Walking,
    Falling,
    Gliding,
}

#[derive(Default, Debug, Copy, Clone)]
pub enum CameraMode {
    #[default]
    ThirdPerson,
    FirstPerson,
}

impl CameraMode {
    pub fn toggle(&mut self) {
        *self = match *self {
            CameraMode::ThirdPerson => CameraMode::FirstPerson,
            CameraMode::FirstPerson => CameraMode::ThirdPerson,
        };
    }
}

impl Default for MovementState {
    fn default() -> Self {
        Self::Walking
    }
}

#[derive(Default, Debug, Component)]
#[uuid = "fc8bd668-fc0a-4ab7-8b3d-f0f22bb539e2"]
pub struct MovementComponent {
    pub velocity: Vec3,
    pub is_falling: bool,
    pub is_flying: bool,
    pub view: Quat,
}

#[derive(Default, Debug, Component)]
#[uuid = "8d2df877-499b-46f3-9660-bd2e1867af0d"]
pub struct CameraComponent {
    pub x: f32,
    pub y: f32,
    pub current_x: f32,
    pub current_y: f32,
    #[reflect(skip)]
    pub mode: CameraMode,
}

#[derive(Default, Debug, Component)]
#[uuid = "ac41cdd4-3311-45ef-815c-9a31adbe4098"]
pub struct CharacterControllerComponent {
    pub horizontal_velocity: Vec3,
    pub vertical_velocity: Vec3,
    pub camera_view: Quat,
    #[reflect(skip)]
    pub movement_state: MovementState,
    pub visual_rotation: Quat,
    pub fall_time: f32,
}

#[derive(Debug, Component)]
#[uuid = "16ca6de6-7a30-412d-8bef-4ee96e18a101"]
pub struct CharacterConfigComponent {
    pub max_movement_speed: f32,
    pub gravity_dir: Vec3,
    pub gravity_strength: f32,
    pub max_walkable_slope: f32,
    pub step_size: f32,
    pub walk_offset: f32,
    pub jump_velocity: f32,
    pub max_gliding_downwards_speed: f32,
    pub gliding_gravity_scale: f32,
}

impl CharacterConfigComponent {
    pub fn is_walkable(&self, normal: Vec3) -> bool {
        Vec3::dot(normal, Vec3::Z) > f32::to_radians(self.max_walkable_slope).cos()
    }
}
impl Default for CharacterConfigComponent {
    fn default() -> Self {
        Self {
            max_movement_speed: 500.0,
            gravity_dir: -Vec3::Z,
            gravity_strength: 981.0,
            max_walkable_slope: 50.0,
            step_size: 15.0,
            walk_offset: 2.0,
            jump_velocity: 600.0,
            max_gliding_downwards_speed: 100.0,
            gliding_gravity_scale: 0.2,
        }
    }
}

pub struct MovementLog;
impl MovementLog {
    pub const STEP_UP: LogCategory = LogCategory::new("StepUp");
    pub const PENETRATION: LogCategory = LogCategory::new("Penetration");
}

pub struct PlayerInput;
impl PlayerInput {
    pub const MOVE_FORWARD: &'static str = "MoveForward";
    pub const MOVE_RIGHT: &'static str = "MoveRight";
    pub const LOOK_UP: &'static str = "LookUp";
    pub const TURN_RIGHT: &'static str = "TurnRight";
    pub const TOGGLE_CAMERA: &'static str = "ToggleCamera";
    pub const JUMP: &'static str = "Jump";
}
fn register_class_resource(mut commands: Commands) {
    let mut len: usize = 0;
    unsafe {
        (bindings().get_registered_classes)(std::ptr::null_mut(), &mut len);
    }
    let mut classes: Vec<*mut UClassOpague> = Vec::with_capacity(len);
    unsafe {
        (bindings().get_registered_classes)(classes.as_mut_ptr(), &mut len);
        classes.set_len(len);
    }

    let mut classes_resource = ClassesResource::default();

    for (id, class_ptr) in classes.into_iter().enumerate() {
        log::info!("register {:?} {:?}", id, class_ptr);
        if let Some(class) = Class::from(id as u32) {
            classes_resource.classes.insert(class_ptr, class);
        }
    }
    commands.insert_resource(classes_resource);
}

fn spawn_class(
    class_resource: Res<ClassesResource>,
    query: Query<(Entity, &ActorComponent), Added<ActorComponent>>,
    mut commands: Commands,
) {
    for (entity, actor) in query.iter() {
        unsafe {
            let class_ptr = (bindings().get_class)(actor.actor.0);
            if let Some(&class) = class_resource.classes.get(&class_ptr) {
                match class {
                    Class::Player => {
                        commands.entity(entity).insert_bundle((
                            CharacterConfigComponent::default(),
                            CharacterControllerComponent::default(),
                            MovementComponent::default(),
                        ));
                    }
                }
            }
        }
    }
}

fn register_player_input(mut input: ResMut<Input>) {
    input.register_axis_binding(PlayerInput::MOVE_FORWARD);
    input.register_axis_binding(PlayerInput::MOVE_RIGHT);
    input.register_axis_binding(PlayerInput::LOOK_UP);
    input.register_axis_binding(PlayerInput::TURN_RIGHT);
    input.register_action_binding(PlayerInput::JUMP);
    input.register_action_binding(PlayerInput::TOGGLE_CAMERA);
}

pub enum MovementHit {
    Slope { normal: Vec3 },
    Wall { normal: Vec3 },
}

pub struct FloorHit {
    pub impact_location: Vec3,
}

pub fn resolve_possible_penetration(
    actor: &ActorComponent,
    transform: &TransformComponent,
    physics: &PhysicsComponent,
) -> Option<Vec3> {
    let mut params = SweepParams::default();
    params.ignored_actors.push(actor.actor.0);
    let shape = physics.get_collision_shape();
    sweep(
        transform.position,
        transform.position + Vec3::Z * 10.0,
        transform.rotation,
        shape,
        params.clone(),
    )
    .and_then(|hit| {
        if hit.start_in_penentration {
            let new_location = hit.location + hit.normal * (hit.penetration_depth + 2.0);
            unreal_api::log::visual_log_location(
                MovementLog::PENETRATION,
                actor.actor,
                hit.location,
                10.0,
                ffi::Color::RED,
            );
            unreal_api::log::visual_log_location(
                MovementLog::PENETRATION,
                actor.actor,
                new_location,
                10.0,
                ffi::Color::GREEN,
            );

            Some(new_location)
        } else {
            None
        }
    })
}
pub struct StepUpResult {
    location: Vec3,
    impact_location: Vec3,
}

pub fn find_floor(
    actor: &ActorComponent,
    transform: &TransformComponent,
    physics: &PhysicsComponent,
    config: &CharacterConfigComponent,
) -> Option<FloorHit> {
    let mut params = SweepParams::default();
    params.ignored_actors.push(actor.actor.0);
    let shape = physics.get_collision_shape();
    sweep(
        transform.position,
        transform.position + config.gravity_dir * 500.0,
        transform.rotation,
        // we scale the physics shape down to avoid collision with nearby walls
        shape.scale(0.9),
        params,
    )
    .and_then(|hit| {
        let is_walkable = config.is_walkable(hit.impact_normal);

        let is_within_range =
            config.step_size + hit.impact_location.z >= transform.position.z - shape.extent().z;
        if is_walkable && is_within_range {
            Some(FloorHit {
                impact_location: hit.impact_location,
            })
        } else {
            None
        }
    })
}

pub fn movement_hit(
    actor: &ActorComponent,
    transform: &TransformComponent,
    physics: &PhysicsComponent,
    config: &CharacterConfigComponent,
    velocity: Vec3,
    dt: f32,
) -> Option<MovementHit> {
    let mut params = SweepParams::default();

    params.ignored_actors.push(actor.actor.0);
    if let Some(hit) = sweep(
        transform.position,
        transform.position + velocity * dt,
        transform.rotation,
        physics.get_collision_shape(),
        params,
    ) {
        let is_moving_against = Vec3::dot(hit.impact_normal, velocity) < 0.0;

        let is_walkable = config.is_walkable(hit.impact_normal);
        if is_walkable && is_moving_against {
            return Some(MovementHit::Slope {
                normal: hit.impact_normal,
            });
        } else {
            return Some(MovementHit::Wall {
                normal: hit.impact_normal,
            });
        }
    }
    None
}
fn update_movement_component(
    mut query: Query<(&CharacterControllerComponent, &mut MovementComponent)>,
) {
    for (controller, mut movement) in query.iter_mut() {
        movement.velocity = controller.horizontal_velocity + controller.vertical_velocity;
        movement.is_falling = matches!(controller.movement_state, MovementState::Falling);
        movement.is_flying = matches!(controller.movement_state, MovementState::Gliding);
    }
}

fn toggle_camera(
    input: Res<Input>,
    mut camera_query: Query<(Entity, &mut CameraComponent, &ParentComponent)>,
    mut actor_query: Query<&mut ActorComponent>,
) {
    if input.is_action_pressed(PlayerInput::TOGGLE_CAMERA) {
        for (entity, mut camera, parent) in camera_query.iter_mut() {
            camera.mode.toggle();
            if let Ok([camera_actor, mut parent_actor]) =
                actor_query.get_many_mut([entity, parent.parent])
            {
                match camera.mode {
                    CameraMode::FirstPerson => parent_actor.set_owner(Some(&camera_actor)),
                    CameraMode::ThirdPerson => parent_actor.set_owner(None),
                };
            }
        }
    }
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
        let mut input_dir = controller.camera_view * player_input;
        input_dir.z = 0.0;
        controller.horizontal_velocity = input_dir.normalize_or_zero() * config.max_movement_speed;

        if let Some(new_position) = resolve_possible_penetration(actor, &transform, physics) {
            transform.position = new_position;
        }

        let new_state = match controller.movement_state {
            MovementState::Walking => {
                controller.fall_time = 0.0;
                if let Some(hit) = find_floor(actor, &transform, physics, config) {
                    //transform.position = hit.position;
                    controller.vertical_velocity = Vec3::ZERO;
                    transform.position.z = hit.impact_location.z
                        + physics.get_collision_shape().extent().z
                        + config.walk_offset;

                    if input.is_action_pressed(PlayerInput::JUMP) {
                        controller.vertical_velocity.z += config.jump_velocity;
                        Some(MovementState::Falling)
                    } else {
                        None
                    }
                } else {
                    Some(MovementState::Falling)
                }
            }
            MovementState::Falling => {
                controller.fall_time += frame.dt;

                let is_downwards = controller.vertical_velocity.z < 0.0;
                if find_floor(actor, &transform, physics, config).is_some() && is_downwards {
                    Some(MovementState::Walking)
                } else if input.is_action_pressed(PlayerInput::JUMP) {
                    Some(MovementState::Gliding)
                } else {
                    controller.vertical_velocity +=
                        config.gravity_dir * config.gravity_strength * frame.dt;
                    None
                }
            }
            MovementState::Gliding => {
                let is_downwards = controller.vertical_velocity.z < 0.0;

                //let mut fwd = controller.camera_view * player_input;
                if find_floor(actor, &transform, physics, config).is_some() && is_downwards {
                    Some(MovementState::Walking)
                } else if input.is_action_pressed(PlayerInput::JUMP) {
                    Some(MovementState::Falling)
                } else {
                    controller.vertical_velocity += config.gravity_dir
                        * config.gravity_strength
                        * config.gliding_gravity_scale
                        * frame.dt;

                    controller.vertical_velocity.z = f32::max(
                        -config.max_gliding_downwards_speed,
                        controller.vertical_velocity.z,
                    );

                    None
                }
            }
        };

        if let Some(new_state) = new_state {
            controller.movement_state = new_state;
        }

        if let Some(hit) = movement_hit(
            actor,
            &transform,
            physics,
            config,
            controller.horizontal_velocity,
            frame.dt,
        ) {
            match hit {
                MovementHit::Slope { normal } => {
                    controller.horizontal_velocity =
                        project_onto_plane(controller.horizontal_velocity, normal)
                            .normalize_or_zero()
                            * config.max_movement_speed
                }
                MovementHit::Wall { normal } => {
                    let is_wall = Vec3::dot(normal, Vec3::Z) < 0.2;

                    if is_wall {
                        let mut params = SweepParams::default();
                        params.ignored_actors.push(actor.actor.0);

                        let target_pos = transform.position
                            + controller.horizontal_velocity.normalize_or_zero() * 5.0
                            + Vec3::Z * (config.step_size + 10.0);

                        if let Some(step_result) = sweep(
                            target_pos,
                            target_pos - Vec3::Z * 100.0,
                            transform.rotation,
                            physics.get_collision_shape(),
                            params,
                        )
                        .and_then(|hit| {
                            if hit.start_in_penentration {
                                return None;
                            }
                            if hit.impact_location.z - transform.position.z < config.step_size / 2.0
                            {
                                Some(StepUpResult {
                                    location: hit.location,
                                    impact_location: hit.impact_location,
                                })
                            } else {
                                None
                            }
                        }) {
                            let shape = physics.get_collision_shape();
                            unreal_api::log::visual_log_shape(
                                MovementLog::STEP_UP,
                                actor.actor,
                                transform.position,
                                transform.rotation,
                                shape,
                                ffi::Color::BLUE,
                            );

                            unreal_api::log::visual_log_shape(
                                MovementLog::STEP_UP,
                                actor.actor,
                                step_result.location,
                                transform.rotation,
                                shape,
                                ffi::Color::RED,
                            );
                            unreal_api::log::visual_log_location(
                                MovementLog::STEP_UP,
                                actor.actor,
                                step_result.impact_location,
                                5.0,
                                ffi::Color::GREEN,
                            );
                            transform.position = step_result.location + Vec3::Z * 2.0;
                        } else {
                            let wall_normal = normal.xy().extend(0.0).normalize_or_zero();

                            controller.horizontal_velocity =
                                project_onto_plane(controller.horizontal_velocity, wall_normal)
                        }
                    } else {
                        let wall_normal = normal.xy().extend(0.0).normalize_or_zero();

                        controller.horizontal_velocity =
                            project_onto_plane(controller.horizontal_velocity, wall_normal)
                    }
                }
            }
        }
        {
            let mut params = SweepParams::default();

            params.ignored_actors.push(actor.actor.0);
            if let Some(hit) = sweep(
                transform.position,
                transform.position + controller.vertical_velocity * frame.dt,
                transform.rotation,
                physics.get_collision_shape(),
                params,
            ) {
                // Lazy: lets just set the vertical_velocity to zero if we hit something
                if !hit.start_in_penentration {
                    controller.vertical_velocity = Vec3::ZERO;
                }
            }
        }
        transform.position +=
            (controller.horizontal_velocity + controller.vertical_velocity) * frame.dt;

        if controller.horizontal_velocity.length() > 0.2 {
            let velocity_dir = controller.horizontal_velocity.normalize_or_zero();
            let target_rot = Quat::from_rotation_z(f32::atan2(velocity_dir.y, velocity_dir.x));
            transform.rotation = Quat::lerp(transform.rotation, target_rot, frame.dt * 10.0);
        }
    }
}

fn update_controller_view(
    mut movement: Query<&mut CharacterControllerComponent>,
    camera: Query<(&ParentComponent, &TransformComponent, &CameraComponent)>,
) {
    for (parent, spatial, _) in camera.iter() {
        if let Ok(mut movement) =
            movement.get_component_mut::<CharacterControllerComponent>(parent.parent)
        {
            movement.camera_view = spatial.rotation;
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
    unsafe {
        (bindings().get_mouse_delta)(&mut x, &mut y);
    }

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
    mut query: Query<(Entity, &ActorComponent, Added<CharacterControllerComponent>)>,
) {
    for (entity, _, added) in query.iter_mut() {
        if !added {
            continue;
        }
        let pos = Vec3::new(-2587.0, -1800.0, 150.0);
        unsafe {
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
                    actor: ActorPtr(actor),
                },
                CameraComponent::default(),
                ParentComponent { parent: entity },
            ));
        }
    }
}

fn update_camera(
    mut query: Query<(Entity, &ParentComponent, &CameraComponent)>,
    mut spatial_query: Query<&mut TransformComponent>,
) {
    for (entity, parent, camera) in query.iter_mut() {
        let spatial_parent = spatial_query
            .get_component::<TransformComponent>(parent.parent)
            .ok()
            .cloned();
        let spatial = spatial_query
            .get_component_mut::<TransformComponent>(entity)
            .ok();
        if let (Some(mut spatial), Some(parent)) = (spatial, spatial_parent) {
            let local_offset = match camera.mode {
                CameraMode::ThirdPerson => spatial.rotation * Vec3::new(-500.0, 0.0, 150.0),
                CameraMode::FirstPerson => Vec3::new(0.0, 0.0, 50.0),
            };

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
    fn initialize(&self, module: &mut Module) {
        register_components2! {
            CameraComponent,
            MovementComponent,
            CharacterConfigComponent,
            => module
        };

        module
            .add_startup_system_set(
                SystemSet::new()
                    .with_system(register_class_resource)
                    .with_system(register_player_input),
            )
            .add_system_set_to_stage(
                CoreStage::Update,
                SystemSet::new()
                    .with_system(spawn_class)
                    .with_system(spawn_camera)
                    .with_system(character_control_system)
                    .with_system(update_controller_view)
                    .with_system(update_movement_component.after(character_control_system))
                    .with_system(rotate_camera)
                    .with_system(update_camera.after(rotate_camera))
                    .with_system(toggle_camera),
            );
    }
}

unreal_api::implement_unreal_module!(MyModule);
