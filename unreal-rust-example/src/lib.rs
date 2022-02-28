use std::collections::HashMap;

use bevy_ecs::prelude::*;
use unreal_api::{
    core::{
        ActorComponent, ActorPtr, CameraComponent, CoreStage, Frame, MovementComponent,
        ParentComponent, PhysicsComponent, TransformComponent,
    },
    ffi::{self, UClassOpague},
    input::Input,
    math::{Quat, Vec3},
    module::{bindings, InitUserModule, UserModule},
    physics::{line_trace, sweep, SweepParams},
};
use unreal_reflect::{impl_component, registry::ReflectionRegistry, TypeUuid};

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
#[derive(Debug, TypeUuid)]
#[uuid = "16ca6de6-7a30-412d-8bef-4ee96e18a101"]
pub struct CharacterConfigComponent {
    pub max_movement_speed: f32,
    pub gravity_dir: Vec3,
    pub gravity_strength: f32,
    pub max_walkable_slope: f32,
    pub max_height_until_fall: f32,
}
impl Default for CharacterConfigComponent {
    fn default() -> Self {
        Self {
            max_movement_speed: 500.0,
            gravity_dir: -Vec3::Z,
            gravity_strength: 981.0,
            max_walkable_slope: 50.0,
            max_height_until_fall: 15.0,
        }
    }
}
impl_component!(CharacterConfigComponent);

pub struct PlayerInput;
impl PlayerInput {
    pub const MOVE_FORWARD: &'static str = "MoveForward";
    pub const MOVE_RIGHT: &'static str = "MoveRight";
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
        log::info!("{:?} {:?}", id, class_ptr);
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
            let class_ptr = (bindings().get_class)(actor.ptr.0);
            if let Some(&class) = class_resource.classes.get(&class_ptr) {
                match class {
                    Class::Player => {
                        log::info!("PLAYER");
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
}

pub enum MovementHit {
    Slope { normal: Vec3 },
    Wall { normal: Vec3 },
}

pub struct FloorHit {
    pub position: Vec3,
}
pub fn find_floor(
    actor: &ActorComponent,
    transform: &TransformComponent,
    _physics: &PhysicsComponent,
    config: &CharacterConfigComponent,
) -> Option<FloorHit> {
    let mut params = SweepParams::default();

    params.ignored_actors.push(actor.ptr.0);
    if let Some(hit) = line_trace(
        transform.position,
        transform.position + config.gravity_dir * 1000.0,
        params,
    ) {
        let distance = Vec3::distance(transform.position, hit.impact_location) - 110.0;
        if Vec3::dot(hit.normal, Vec3::Z) > 0.9 && distance <= config.max_height_until_fall {
            return Some(FloorHit {
                position: hit.impact_location,
            });
        }
    }
    None
}

pub fn movement_hit(
    actor: &ActorComponent,
    transform: &TransformComponent,
    physics: &PhysicsComponent,
    _config: &CharacterConfigComponent,
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
    None
}
fn update_movement_velocity(
    mut query: Query<(&CharacterControllerComponent, &mut MovementComponent)>,
) {
    for (controller, mut movement) in query.iter_mut() {
        movement.velocity = controller.velocity;
    }
}

fn add_character_controller(
    query: Query<(Entity, &ActorComponent, &PhysicsComponent)>,
    mut commands: Commands,
) {
    for (entity, _, _) in query.iter() {
        commands.entity(entity).insert_bundle((
            CharacterConfigComponent::default(),
            CharacterControllerComponent::default(),
            MovementComponent::default(),
        ));
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
        input_dir = input_dir.normalize_or_zero() * config.max_movement_speed;
        input_dir.z = controller.velocity.z;
        controller.velocity = input_dir;

        if let Some(_hit) = find_floor(actor, &transform, physics, config) {
            //transform.position = hit.position;
            controller.velocity.z = 0.0;
        } else {
            controller.velocity += config.gravity_dir * config.gravity_strength * frame.dt;
        }
        if let Some(hit) = movement_hit(actor, &transform, physics, config, &controller, frame.dt) {
            match hit {
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
        transform.position += controller.velocity * frame.dt * 1.0;

        if controller.velocity.length() > 0.2 {
            let velocity_dir = controller.velocity.normalize_or_zero();
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
                    ptr: ActorPtr(actor),
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
        startup.add_system_to_stage(CoreStage::Startup, register_class_resource.system());
        startup.add_system_to_stage(CoreStage::Startup, register_player_input.system());
        update.add_system_to_stage(CoreStage::Update, spawn_class.system());
        update.add_system_to_stage(CoreStage::Update, spawn_camera.system());
        update.add_system_to_stage(CoreStage::Update, character_control_system.system());
        update.add_system_to_stage(CoreStage::Update, update_controller_view.system());
        update.add_system_to_stage(CoreStage::Update, update_movement_velocity.system());
        update.add_system_to_stage(CoreStage::Update, rotate_camera.system());
        update.add_system_to_stage(CoreStage::Update, update_camera.system());
    }
}

unreal_api::implement_unreal_module!(MyModule);
