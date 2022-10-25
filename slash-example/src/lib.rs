use std::collections::HashMap;

use bevy_ecs::prelude::*;
use unreal_api::api::{SweepParams, UnrealApi};
use unreal_api::core::{ActorHitEvent, Despawn, Frame};
use unreal_api::editor_component::InsertSerializedComponent;
use unreal_api::ffi::Sweep;
use unreal_api::math::Vec2;
use unreal_api::physics::PhysicsComponent;
use unreal_api::registry::{UClass, USound};
use unreal_api::sound::{play_sound_at_location, SoundSettings};
use unreal_api::{
    core::{ActorComponent, ActorPtr, CoreStage, ParentComponent, TransformComponent},
    ffi::{self, UClassOpague},
    input::Input,
    math::{Quat, Vec3},
    module::{bindings, InitUserModule, Module, UserModule},
    register_components,
};
use unreal_api::{Component, TypeUuid};
use unreal_movement::{
    CharacterConfigComponent, CharacterControllerComponent, MovementComponent, MovementPlugin,
};

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

pub struct PlayerInput;
impl PlayerInput {
    pub const MOVE_FORWARD: &'static str = "MoveForward";
    pub const MOVE_RIGHT: &'static str = "MoveRight";
    pub const LOOK_UP: &'static str = "LookUp";
    pub const TURN_RIGHT: &'static str = "TurnRight";
    pub const TOGGLE_CAMERA: &'static str = "ToggleCamera";
    pub const JUMP: &'static str = "Jump";
    pub const ROTATE_CAMERA: &'static str = "RotateCamera";
    pub const PRIMARY_ATTACK: &'static str = "PrimaryAttack";
}

#[derive(Component, serde::Deserialize)]
#[uuid = "cdbbc147-058c-4079-aa85-a93a30d1edfc"]
#[reflect(editor)]
pub struct PlayerComponent {
    pub actor: UClass,
}

impl InsertSerializedComponent for PlayerComponentReflect {
    unsafe fn insert_serialized_component(
        &self,
        json: &str,
        commands: &mut bevy_ecs::system::EntityCommands<'_, '_, '_>,
    ) {
        let component = serde_json::de::from_str::<PlayerComponent>(json).unwrap();
        commands.insert(component);
    }
}

#[derive(Component, serde::Serialize, serde::Deserialize)]
#[uuid = "e9815aea-f4e2-4953-9553-079e6a7b8055"]
#[reflect(editor)]
pub struct WeaponComponent {}

#[derive(Default, Component)]
#[uuid = "065f42a3-1925-4305-be29-9bb60a4ba510"]
pub struct HeroComponent {
    pub is_attacking: bool,
    #[reflect(skip)]
    pub weapon: Option<Entity>,
}

#[derive(Component)]
#[uuid = "26e67b30-9dc4-4a57-92e0-00e5f49ef18c"]
pub struct TopdownCameraComponent {
    #[reflect(skip)]
    pub target: Entity,
    pub rotation: f32,
    pub target_position: Vec3,
}

#[derive(Default, Component)]
#[uuid = "2f2b4a61-0d7b-45a8-a42d-7b2af69598be"]
pub struct CursorComponent {
    // TODO: should be vec2
    pub position: Vec3,
    pub is_visible: bool,
}
#[derive(Default, Component)]
#[uuid = "85165d56-db4b-471d-a346-bd13287f4d88"]
pub struct PlayerStateComponent {
    pub cursor_position: Vec3,
    pub is_cursor_visible: bool,
}

fn create_player(
    query: Query<(Entity, &ActorComponent), Added<PlayerComponent>>,
    mut commands: Commands,
) {
    for (entity, actor) in query.iter() {
        unsafe {
            (bindings().actor_fns.register_actor_on_overlap)(actor.actor.0);
        }
        unsafe {
            (bindings().actor_fns.register_actor_on_hit)(actor.actor.0);
        }
        commands.entity(entity).insert_bundle((
            CharacterConfigComponent::default(),
            CharacterControllerComponent::default(),
            MovementComponent::default(),
            PlayerStateComponent::default(),
            CursorComponent::default(),
            HeroComponent::default(),
        ));
    }
}

fn register_player_input(mut input: ResMut<Input>) {
    input.register_axis_binding(PlayerInput::MOVE_FORWARD);
    input.register_axis_binding(PlayerInput::MOVE_RIGHT);
    input.register_action_binding(PlayerInput::ROTATE_CAMERA);
    input.register_action_binding(PlayerInput::PRIMARY_ATTACK);
}

fn player_attack(input: Res<Input>, mut query: Query<&mut HeroComponent>) {
    let is_attacking = input.is_action_pressed(PlayerInput::PRIMARY_ATTACK);

    for mut hero in &mut query {
        hero.is_attacking = is_attacking;
    }
}

fn register_weapon(
    api: Res<UnrealApi>,
    mut hero: Query<&mut HeroComponent>,
    query: Query<(Entity, &ActorComponent, Added<WeaponComponent>)>,
) {
    for (entity, actor, added) in &query {
        if !added {
            if let Some(parent) = actor.get_parent(&api) {
                if let Ok(mut hero) = hero.get_mut(parent) {
                    hero.weapon = Some(entity);
                }
            }
        }
    }
}

fn apply_weapon_forces(
    api: Res<UnrealApi>,
    query: Query<(Entity, &HeroComponent)>,
    weapon: Query<(
        Entity,
        &WeaponComponent,
        &PhysicsComponent,
        &TransformComponent,
    )>,
    mut physics_query: Query<(
        &mut PhysicsComponent,
        &ActorComponent,
        Without<WeaponComponent>,
    )>,
) {
    for (entity, hero) in &query {
        if let Ok((weapon_entity, _, p, transform)) = weapon.get(hero.weapon.unwrap()) {
            let params = SweepParams::default()
                .add_ignored_entity(entity)
                .add_ignored_entity(weapon_entity);
            let result = api.overlap_multi(
                transform.position,
                transform.rotation,
                p.get_collision_shape(),
                params,
                2,
            );

            for entity in result {
                match physics_query.get_mut(entity) {
                    Ok((mut physics, actor, _)) => {
                        log::info!(
                            "test {} {}",
                            actor.get_actor_name(),
                            physics.ptr.ptr as usize
                        );
                        if physics.is_simulating {
                            log::info!("hit {}", actor.get_actor_name());
                            physics.add_impulse(Vec3::Z * 500000.0);
                        }
                    }
                    Err(err) => log::info!("{}", err),
                }
            }
        }
    }
}
fn update_player_state(
    input: Res<Input>,
    mut query: Query<(&CursorComponent, &mut PlayerStateComponent)>,
) {
    for (cursor, mut state) in &mut query {
        state.is_cursor_visible = !input.is_action_pressed(PlayerInput::ROTATE_CAMERA);
        state.cursor_position = cursor.position;
    }
}

fn update_cursor(input: Res<Input>, mut cursor: Query<&mut CursorComponent>) {
    for mut cursor in &mut cursor {
        if !input.is_action_pressed(PlayerInput::ROTATE_CAMERA) {
            let (x, y) = input.get_mouse_delta();
            cursor.position += Vec3::new(x, -y, 0.0) * 15.0;
        }
    }
}
fn rotate_camera(
    input: Res<Input>,
    mut topdown: Query<&mut TopdownCameraComponent>,
    mut character: Query<&mut TransformComponent>,
    mut cursor: Query<&mut CursorComponent>,
) {
    if input.is_action_pressed(PlayerInput::ROTATE_CAMERA) {
        //unsafe { (bindings().viewport_fns.set_mouse_state)(0, ffi::MouseState::Hidden) };
    }
    if input.is_action_released(PlayerInput::ROTATE_CAMERA) {
        //unsafe { (bindings().viewport_fns.set_mouse_state)(0, ffi::MouseState::Hidden) };
    }

    unsafe {
        //(bindings().viewport_fns.get_mouse_position)(0, &mut x, &mut y);
    };

    for mut cam in &mut topdown {
        let mouse = cursor
            .get(cam.target)
            .map_or(Vec3::ZERO, |cursor| cursor.position);

        let mut screen_x = 0.0f32;
        let mut screen_y = 0.0f32;
        unsafe {
            (bindings().viewport_fns.get_viewport_size)(0, &mut screen_x, &mut screen_y);
        };

        let mut dir = Vec2::new(mouse.x / screen_x, mouse.y / screen_y)
            .mul_add(Vec2::splat(2.0), Vec2::splat(-1.0))
            .normalize_or_zero();

        dir.y *= -1.0;

        let mut angle = f32::acos(Vec2::dot(dir, Vec2::Y));
        if dir.x < 0.0 {
            angle = 2.0 * std::f32::consts::PI - angle;
        }

        let rotation = if input.is_action_pressed(PlayerInput::ROTATE_CAMERA) {
            let (x, _) = input.get_mouse_delta();
            cam.rotation += x * 0.03;
            Quat::from_rotation_z(cam.rotation)
        } else {
            Quat::from_rotation_z(cam.rotation + angle)
        };

        if let Ok(mut transform) = character.get_mut(cam.target) {
            transform.rotation = Quat::slerp(transform.rotation, rotation, 0.2);
        }
    }
}
fn update_controller_view(
    mut movement: Query<&mut CharacterControllerComponent>,
    topdown: Query<&TopdownCameraComponent>,
) {
    for cam in &topdown {
        if let Ok(mut controller) =
            movement.get_component_mut::<CharacterControllerComponent>(cam.target)
        {
            controller.camera_view = Quat::from_rotation_z(cam.rotation);
        }
    }
}

fn update_camera(
    mut topdown: Query<(Entity, &mut TopdownCameraComponent)>,
    mut transform: Query<&mut TransformComponent>,
) {
    let height = 650.0;
    let radius = 350.0;
    let angle = f32::to_radians(90.0) - f32::atan(radius / height);

    for (entity, mut topdown) in topdown.iter_mut() {
        let rotation = Quat::from_rotation_z(topdown.rotation);

        let camera_position = rotation * Vec3::new(-radius, 0.0, height);
        let camera_rotation = rotation * Quat::from_rotation_y(angle);
        if let Ok([mut camera_trans, target_trans]) =
            transform.get_many_mut([entity, topdown.target])
        {
            topdown.target_position =
                Vec3::lerp(topdown.target_position, target_trans.position, 0.2);
            camera_trans.position = topdown.target_position + camera_position;
            camera_trans.rotation = camera_rotation;
        }
    }
}

fn spawn_camera(
    mut commands: Commands,
    mut query: Query<(Entity, &TransformComponent, Added<PlayerComponent>)>,
) {
    for (entity, transform, added) in query.iter_mut() {
        if !added {
            continue;
        }

        unsafe {
            let actor = (bindings().spawn_actor)(
                ffi::ActorClass::CameraActor,
                // TODO: Compute correct location
                (transform.position + Vec3::Z * 200.0).into(),
                Quat::from_rotation_x(0.0).into(),
                Vec3::ONE.into(),
            );
            (bindings().actor_fns.set_view_target)(actor);
            commands.spawn().insert_bundle((
                TransformComponent {
                    position: transform.position + Vec3::Z * 200.0,
                    rotation: Quat::from_rotation_y(f32::to_radians(60.0)),
                    ..Default::default()
                },
                ActorComponent {
                    actor: ActorPtr(actor),
                },
                TopdownCameraComponent {
                    target: entity,
                    rotation: 0.0,
                    target_position: transform.position,
                },
            ));
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
        register_components! {
            PlayerComponent,
            TopdownCameraComponent,
            CursorComponent,
            PlayerStateComponent,
            HeroComponent,
            WeaponComponent,
            => module
        };

        module
            .add_plugin(MovementPlugin)
            .add_startup_system_set(SystemSet::new().with_system(register_player_input))
            .add_system_set_to_stage(
                CoreStage::Update,
                SystemSet::new()
                    .with_system(register_weapon)
                    .with_system(apply_weapon_forces.after(register_weapon))
                    .with_system(player_attack)
                    .with_system(update_cursor)
                    .with_system(create_player)
                    .with_system(spawn_camera)
                    .with_system(rotate_camera)
                    .with_system(update_camera.after(rotate_camera))
                    .with_system(update_controller_view.after(rotate_camera)),
            )
            .add_system_set_to_stage(
                CoreStage::PostUpdate,
                SystemSet::new().with_system(update_player_state),
            );
    }
}

unreal_api::implement_unreal_module!(MyModule);
