use bevy_ecs::{prelude::*, query::WorldQuery};
use unreal_api::Component;
use unreal_api::{
    core::{ActorComponent, ActorPtr, ActorRegistration, CoreStage, Frame, TransformComponent},
    ffi,
    input::Input,
    log::LogCategory,
    math::{Quat, Vec3, Vec3Swizzles},
    module::Module,
    physics::{sweep, PhysicsComponent, SweepParams, SweepResult},
    plugin::Plugin,
    register_components,
};
fn project_onto_plane(dir: Vec3, normal: Vec3) -> Vec3 {
    dir - normal * Vec3::dot(dir, normal)
}

#[derive(Debug, Copy, Clone)]
pub enum MovementState {
    Walking,
    Falling,
    Gliding,
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
#[uuid = "ac41cdd4-3311-45ef-815c-9a31adbe4098"]
pub struct CharacterControllerComponent {
    pub horizontal_velocity: Vec3,
    pub vertical_velocity: Vec3,
    pub camera_view: Quat,
    #[reflect(skip)]
    pub movement_state: MovementState,
    pub visual_rotation: Quat,
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
    pub ground_offset: f32,
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
            step_size: 65.0,
            walk_offset: 2.0,
            jump_velocity: 600.0,
            max_gliding_downwards_speed: 100.0,
            gliding_gravity_scale: 0.2,
            ground_offset: 2.0,
        }
    }
}

pub struct MovementLog;
impl MovementLog {
    pub const STEP_UP: LogCategory = LogCategory::new("StepUp");
    pub const PENETRATION: LogCategory = LogCategory::new("Penetration");
}
pub enum MovementHit {
    Slope { normal: Vec3 },
    Wall { hit: SweepResult },
}

pub struct FloorHit {
    pub actor: Option<ActorPtr>,
    pub impact_location: Vec3,
}

pub struct StepUpResult {
    location: Vec3,
}

fn do_walking(
    movement: &mut MovementQueryItem,
    input: &Input,
    dt: f32,
    registry: &ActorRegistration,
    query: &Query<&PhysicsComponent>,
) -> Option<MovementState> {
    if let Some(hit) = movement.find_floor() {
        movement.controller.vertical_velocity = Vec3::ZERO;
        movement.transform.position.z = hit.impact_location.z
            + movement.physics.get_collision_shape().extent().z
            + movement.config.walk_offset;

        if input.is_action_pressed(PlayerInput::JUMP) {
            movement.controller.vertical_velocity.z += movement.config.jump_velocity;
            return Some(MovementState::Falling);
        }
        let entity = hit.actor.and_then(|ptr| registry.actor_to_entity.get(&ptr));
        let phys = entity.and_then(|&entity| query.get(entity).ok());
        let velocity = phys.map(|p| p.velocity).unwrap_or_default();
        movement.do_movement(velocity, dt);
        None
    } else {
        Some(MovementState::Falling)
    }
}

fn do_falling(movement: &mut MovementQueryItem, input: &Input, dt: f32) -> Option<MovementState> {
    let is_downwards = movement.controller.vertical_velocity.z < 0.0;
    if movement.find_floor().is_some() && is_downwards {
        Some(MovementState::Walking)
    } else if input.is_action_pressed(PlayerInput::JUMP) {
        Some(MovementState::Gliding)
    } else {
        movement.controller.vertical_velocity +=
            movement.config.gravity_dir * movement.config.gravity_strength * dt;
        movement.do_movement(Vec3::ZERO, dt);
        None
    }
}

fn do_gliding(movement: &mut MovementQueryItem, input: &Input, dt: f32) -> Option<MovementState> {
    let is_downwards = movement.controller.vertical_velocity.z < 0.0;

    if movement.find_floor().is_some() && is_downwards {
        Some(MovementState::Walking)
    } else if input.is_action_pressed(PlayerInput::JUMP) {
        Some(MovementState::Falling)
    } else {
        movement.controller.vertical_velocity += movement.config.gravity_dir
            * movement.config.gravity_strength
            * movement.config.gliding_gravity_scale
            * dt;

        movement.controller.vertical_velocity.z = f32::max(
            -movement.config.max_gliding_downwards_speed,
            movement.controller.vertical_velocity.z,
        );
        movement.do_movement(Vec3::ZERO, dt);

        None
    }
}

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct MovementQuery<'w> {
    actor: &'w ActorComponent,
    transform: &'w mut TransformComponent,
    physics: &'w PhysicsComponent,
    controller: &'w mut CharacterControllerComponent,
    config: &'w CharacterConfigComponent,
}
impl<'w> MovementQueryItem<'w> {
    pub fn try_step_up(&self, move_result: &SweepResult) -> Option<StepUpResult> {
        let params = SweepParams::default()
            // Don't test against ourselves
            .add_ignored_actor(self.actor.actor);

        // We want to slighty offset the start of the sweep in the direction we were moving,
        // otherwise we might miss the downwards sweep test
        let offset = self.controller.horizontal_velocity.normalize_or_zero() * 5.0;

        let sweep_start = move_result.location
            + offset
            + Vec3::Z * (self.config.step_size + self.config.ground_offset);

        sweep(
            sweep_start,
            // We want to sweep downards but no more than we can step
            sweep_start - Vec3::Z * self.config.step_size,
            self.transform.rotation,
            self.physics.get_collision_shape(),
            params,
        )
        .and_then(|hit| {
            // Nope we started in penetration which means the object in front of us is higher than
            // we can step
            // Or there could not be enough space on top of the object we are trying to step on
            if hit.start_in_penentration {
                return None;
            }
            unreal_api::log::visual_log_shape(
                MovementLog::STEP_UP,
                self.actor.actor,
                sweep_start,
                self.transform.rotation,
                self.physics.get_collision_shape(),
                ffi::Color::BLUE,
            );

            unreal_api::log::visual_log_shape(
                MovementLog::STEP_UP,
                self.actor.actor,
                hit.location,
                self.transform.rotation,
                self.physics.get_collision_shape(),
                ffi::Color::RED,
            );

            unreal_api::log::visual_log_location(
                MovementLog::STEP_UP,
                self.actor.actor,
                hit.impact_location,
                5.0,
                ffi::Color::GREEN,
            );
            Some(StepUpResult {
                location: hit.location,
            })
        })
    }
    pub fn resolve_possible_penetration(&self) -> Option<Vec3> {
        let params = SweepParams::default().add_ignored_actor(self.actor.actor);
        let shape = self.physics.get_collision_shape();
        sweep(
            self.transform.position,
            self.transform.position + Vec3::Z * 10.0,
            self.transform.rotation,
            shape,
            params,
        )
        .and_then(|hit| {
            if hit.start_in_penentration {
                let new_location = hit.location + hit.normal * (hit.penetration_depth + 2.0);
                unreal_api::log::visual_log_location(
                    MovementLog::PENETRATION,
                    self.actor.actor,
                    hit.location,
                    10.0,
                    ffi::Color::RED,
                );
                unreal_api::log::visual_log_location(
                    MovementLog::PENETRATION,
                    self.actor.actor,
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

    pub fn do_movement(&mut self, velocity: Vec3, dt: f32) {
        if let Some(hit) = self.movement_hit(self.controller.horizontal_velocity, dt) {
            match hit {
                MovementHit::Slope { normal } => {
                    self.controller.horizontal_velocity =
                        project_onto_plane(self.controller.horizontal_velocity, normal)
                            .normalize_or_zero()
                            * self.config.max_movement_speed
                }
                MovementHit::Wall { hit } => {
                    let is_wall = Vec3::dot(hit.impact_normal, Vec3::Z) < 0.2;

                    if is_wall {
                        if let Some(step_result) = self.try_step_up(&hit) {
                            self.transform.position =
                                step_result.location + Vec3::Z * self.config.ground_offset;
                        } else {
                            let wall_normal =
                                hit.impact_normal.xy().extend(0.0).normalize_or_zero();

                            self.controller.horizontal_velocity =
                                project_onto_plane(self.controller.horizontal_velocity, wall_normal)
                        }
                    } else {
                        let wall_normal = hit.impact_normal.xy().extend(0.0).normalize_or_zero();

                        self.controller.horizontal_velocity =
                            project_onto_plane(self.controller.horizontal_velocity, wall_normal)
                    }
                }
            }
        }
        {
            let params = SweepParams::default().add_ignored_actor(self.actor.actor);

            if let Some(hit) = sweep(
                self.transform.position,
                self.transform.position + self.controller.vertical_velocity * dt,
                self.transform.rotation,
                self.physics.get_collision_shape(),
                params,
            ) {
                // Lazy: lets just set the vertical_velocity to zero if we hit something
                if !hit.start_in_penentration {
                    self.controller.vertical_velocity = Vec3::ZERO;
                }
            }
        }
        self.transform.position +=
            (self.controller.horizontal_velocity + self.controller.vertical_velocity + velocity)
                * dt;

        if self.controller.horizontal_velocity.length() > 0.2 {
            let velocity_dir = self.controller.horizontal_velocity.normalize_or_zero();
            let target_rot = Quat::from_rotation_z(f32::atan2(velocity_dir.y, velocity_dir.x));
            self.transform.rotation = Quat::lerp(self.transform.rotation, target_rot, dt * 10.0);
        }
    }
    pub fn movement_hit(&self, velocity: Vec3, dt: f32) -> Option<MovementHit> {
        let params = SweepParams::default().add_ignored_actor(self.actor.actor);

        if let Some(hit) = sweep(
            self.transform.position,
            self.transform.position + velocity * dt,
            self.transform.rotation,
            self.physics.get_collision_shape(),
            params,
        ) {
            let is_moving_against = Vec3::dot(hit.impact_normal, velocity) < 0.0;

            let is_walkable = self.config.is_walkable(hit.impact_normal);
            if is_walkable && is_moving_against {
                return Some(MovementHit::Slope {
                    normal: hit.impact_normal,
                });
            } else {
                return Some(MovementHit::Wall { hit });
            }
        }
        None
    }

    pub fn find_floor(&self) -> Option<FloorHit> {
        let params = SweepParams::default().add_ignored_actor(self.actor.actor);
        let shape = self.physics.get_collision_shape();
        sweep(
            self.transform.position,
            self.transform.position + self.config.gravity_dir * 500.0,
            self.transform.rotation,
            // we scale the physics shape down to avoid collision with nearby walls
            shape.scale(0.9),
            params,
        )
        .and_then(|hit| {
            let is_walkable = self.config.is_walkable(hit.impact_normal);

            let is_within_range = self.config.step_size + hit.impact_location.z
                >= self.transform.position.z - shape.extent().z;
            if is_walkable && is_within_range {
                Some(FloorHit {
                    actor: hit.actor,
                    impact_location: hit.impact_location,
                })
            } else {
                None
            }
        })
    }
}

fn character_control_system(
    input: Res<Input>,
    frame: Res<Frame>,
    registry: Res<ActorRegistration>,
    mut query: Query<MovementQuery>,
    phys: Query<&PhysicsComponent>,
) {
    let forward = input
        .get_axis_value(PlayerInput::MOVE_FORWARD)
        .unwrap_or(0.0);
    let right = input.get_axis_value(PlayerInput::MOVE_RIGHT).unwrap_or(0.0);
    let player_input = Vec3::new(forward, right, 0.0).normalize_or_zero();

    for mut movement in query.iter_mut() {
        let mut input_dir = movement.controller.camera_view * player_input;
        input_dir.z = 0.0;
        movement.controller.horizontal_velocity =
            input_dir.normalize_or_zero() * movement.config.max_movement_speed;

        if let Some(new_position) = movement.resolve_possible_penetration() {
            movement.transform.position = new_position;
        }

        let new_state = match movement.controller.movement_state {
            MovementState::Walking => do_walking(&mut movement, &input, frame.dt, &registry, &phys),
            MovementState::Falling => do_falling(&mut movement, &input, frame.dt),
            MovementState::Gliding => do_gliding(&mut movement, &input, frame.dt),
        };

        if let Some(new_state) = new_state {
            movement.controller.movement_state = new_state;
        }
    }
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

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, module: &mut Module) {
        register_components! {
            MovementComponent,
            CharacterConfigComponent,
            => module
        };

        module.add_system_set_to_stage(
            CoreStage::Update,
            SystemSet::new()
                .with_system(character_control_system)
                .with_system(update_movement_component.after(character_control_system)),
        );
    }
}
