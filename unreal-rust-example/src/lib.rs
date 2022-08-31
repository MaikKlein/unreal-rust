use std::collections::HashMap;

use bevy_ecs::prelude::*;
use unreal_api::api::UnrealApi;
use unreal_api::core::{ActorHitEvent, Despawn};
use unreal_api::registry::USound;
use unreal_api::sound::{play_sound_at_location, SoundSettings};
use unreal_api::Component;
use unreal_api::{
    core::{ActorComponent, ActorPtr, CoreStage, ParentComponent, TransformComponent},
    ffi::{self, UClassOpague},
    input::Input,
    math::{Quat, Vec3},
    module::{bindings, InitUserModule, Module, UserModule},
    register_components,
};
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

#[derive(Default)]
pub struct ClassesResource {
    classes: HashMap<*mut ffi::UClassOpague, Class>,
}
unsafe impl Send for ClassesResource {}
unsafe impl Sync for ClassesResource {}

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
#[derive(Debug, Component)]
#[uuid = "b6addc7d-03b1-4b06-9328-f26c71997ee6"]
#[reflect(editor)]
pub struct PlaySoundOnImpactComponent {
    pub sound: USound,
}
impl PlaySoundOnImpactComponent {
    const MINIMUM_FORCE: f32 = 30000.0;
}

#[derive(Debug, Component)]
#[uuid = "52788d7e-017b-42cd-b3bf-aa616315c0c4"]
#[reflect(editor)]
pub struct CharacterSoundsComponent {
    pub camera_toggle: USound,
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

pub struct PlayerInput;
impl PlayerInput {
    pub const MOVE_FORWARD: &'static str = "MoveForward";
    pub const MOVE_RIGHT: &'static str = "MoveRight";
    pub const LOOK_UP: &'static str = "LookUp";
    pub const TURN_RIGHT: &'static str = "TurnRight";
    pub const TOGGLE_CAMERA: &'static str = "ToggleCamera";
    pub const JUMP: &'static str = "Jump";
}

// TODO: We probably don't need that anymore
fn register_class_resource(mut commands: Commands) {
    let mut len: usize = 0;
    unsafe {
        (bindings().actor_fns.get_registered_classes)(std::ptr::null_mut(), &mut len);
    }
    let mut classes: Vec<*mut UClassOpague> = Vec::with_capacity(len);
    unsafe {
        (bindings().actor_fns.get_registered_classes)(classes.as_mut_ptr(), &mut len);
        classes.set_len(len);
    }

    let mut classes_resource = ClassesResource::default();

    for (id, class_ptr) in classes.into_iter().enumerate() {
        if let Some(class) = Class::from(id as u32) {
            classes_resource.classes.insert(class_ptr, class);
        }
    }
    commands.insert_resource(classes_resource);
}

fn register_hit_events(mut query: Query<(&mut ActorComponent, Added<PlaySoundOnImpactComponent>)>) {
    for (mut actor, added) in &mut query {
        if added {
            actor.register_on_hit();
        }
    }
}

fn play_sound_on_hit(
    api: Res<UnrealApi>,
    mut events: EventReader<ActorHitEvent>,
    query: Query<(&TransformComponent, &PlaySoundOnImpactComponent)>,
    mut commands: Commands,
) {
    for event in events.iter() {
        if event.normal_impulse.length() <= PlaySoundOnImpactComponent::MINIMUM_FORCE {
            continue;
        }

        if let Some(&entity) = api.actor_to_entity.get(&event.self_actor) {
            if let Ok((trans, sound)) = query.get(entity) {
                play_sound_at_location(
                    sound.sound,
                    trans.position,
                    trans.rotation,
                    &SoundSettings::default(),
                )
            }
            commands.add(Despawn { entity });
        }
    }
}

fn spawn_class(
    class_resource: Res<ClassesResource>,
    query: Query<(Entity, &ActorComponent), Added<ActorComponent>>,
    mut commands: Commands,
) {
    for (entity, actor) in query.iter() {
        unsafe {
            (bindings().actor_fns.register_actor_on_overlap)(actor.actor.0);
        }
        unsafe {
            (bindings().actor_fns.register_actor_on_hit)(actor.actor.0);
        }
        unsafe {
            let class_ptr = (bindings().actor_fns.get_class)(actor.actor.0);
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

fn toggle_camera(
    input: Res<Input>,
    mut camera_query: Query<(Entity, &mut CameraComponent, &ParentComponent)>,
    mut actor_query: Query<&mut ActorComponent>,
    sound: Query<(&TransformComponent, &CharacterSoundsComponent)>,
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

            if let Ok((transform, sound)) = sound.get(parent.parent) {
                play_sound_at_location(
                    sound.camera_toggle,
                    transform.position,
                    transform.rotation,
                    &ffi::SoundSettings::default(),
                )
            }
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

        let smooth = 0.4;
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
            (bindings().actor_fns.set_view_target)(actor);
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
        register_components! {
            CharacterSoundsComponent,
            PlaySoundOnImpactComponent,
            CameraComponent,
            => module

        };

        module
            .add_plugin(MovementPlugin)
            .add_startup_system_set(
                SystemSet::new()
                    .with_system(register_class_resource)
                    .with_system(register_player_input)
                    .with_system(register_hit_events),
            )
            .add_system_set_to_stage(
                CoreStage::Update,
                SystemSet::new()
                    .with_system(spawn_class)
                    .with_system(spawn_camera)
                    .with_system(update_controller_view)
                    .with_system(rotate_camera)
                    .with_system(update_camera.after(rotate_camera))
                    .with_system(toggle_camera)
                    .with_system(play_sound_on_hit),
            );
    }
}

unreal_api::implement_unreal_module!(MyModule);
