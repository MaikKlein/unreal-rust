use std::collections::HashMap;

use bevy_ecs::prelude::*;
use unreal_api::api::UnrealApi;
use unreal_api::core::{ActorHitEvent, Despawn, Frame};
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

pub struct PlayerInput;
impl PlayerInput {
    pub const MOVE_FORWARD: &'static str = "MoveForward";
    pub const MOVE_RIGHT: &'static str = "MoveRight";
    pub const LOOK_UP: &'static str = "LookUp";
    pub const TURN_RIGHT: &'static str = "TurnRight";
    pub const TOGGLE_CAMERA: &'static str = "ToggleCamera";
    pub const JUMP: &'static str = "Jump";
    pub const ROTATE_CAMERA: &'static str = "RotateCamera";
}

#[derive(Component)]
#[uuid = "cdbbc147-058c-4079-aa85-a93a30d1edfc"]
#[reflect(editor)]
pub struct PlayerComponent;

#[derive(Component)]
#[uuid = "26e67b30-9dc4-4a57-92e0-00e5f49ef18c"]
pub struct TopdownCameraComponent {
    #[reflect(skip)]
    pub target: Entity,
    pub rotation: f32,
    pub target_position: Vec3,
}

//// TODO: We probably don't need that anymore
//fn register_class_resource(mut commands: Commands) {
//    let mut len: usize = 0;
//    unsafe {
//        (bindings().actor_fns.get_registered_classes)(std::ptr::null_mut(), &mut len);
//    }
//    let mut classes: Vec<*mut UClassOpague> = Vec::with_capacity(len);
//    unsafe {
//        (bindings().actor_fns.get_registered_classes)(classes.as_mut_ptr(), &mut len);
//        classes.set_len(len);
//    }
//
//    let mut classes_resource = ClassesResource::default();
//
//    for (id, class_ptr) in classes.into_iter().enumerate() {
//        if let Some(class) = Class::from(id as u32) {
//            classes_resource.classes.insert(class_ptr, class);
//        }
//    }
//    commands.insert_resource(classes_resource);
//}

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
        unsafe {
            commands.entity(entity).insert_bundle((
                CharacterConfigComponent::default(),
                CharacterControllerComponent::default(),
                MovementComponent::default(),
            ));
        }
    }
}
//fn spawn_class(
//    class_resource: Res<ClassesResource>,
//    query: Query<(Entity, &ActorComponent), Added<ActorComponent>>,
//    mut commands: Commands,
//) {
//    for (entity, actor) in query.iter() {
//        unsafe {
//            (bindings().actor_fns.register_actor_on_overlap)(actor.actor.0);
//        }
//        unsafe {
//            (bindings().actor_fns.register_actor_on_hit)(actor.actor.0);
//        }
//        unsafe {
//            let class_ptr = (bindings().actor_fns.get_class)(actor.actor.0);
//            if let Some(&class) = class_resource.classes.get(&class_ptr) {
//                match class {
//                    Class::Player => {
//                        commands.entity(entity).insert_bundle((
//                            CharacterConfigComponent::default(),
//                            CharacterControllerComponent::default(),
//                            MovementComponent::default(),
//                        ));
//                    }
//                }
//            }
//        }
//    }
//}

fn register_player_input(mut input: ResMut<Input>) {
    input.register_axis_binding(PlayerInput::MOVE_FORWARD);
    input.register_axis_binding(PlayerInput::MOVE_RIGHT);

    input.register_action_binding(PlayerInput::ROTATE_CAMERA);
    //input.register_axis_binding(PlayerInput::LOOK_UP);
    //input.register_axis_binding(PlayerInput::TURN_RIGHT);
    //input.register_action_binding(PlayerInput::JUMP);
}

//fn toggle_camera(
//    input: Res<Input>,
//    mut camera_query: Query<(Entity, &mut CameraComponent, &ParentComponent)>,
//    mut actor_query: Query<&mut ActorComponent>,
//    sound: Query<(&TransformComponent, &CharacterSoundsComponent)>,
//) {
//    if input.is_action_pressed(PlayerInput::TOGGLE_CAMERA) {
//        for (entity, mut camera, parent) in camera_query.iter_mut() {
//            camera.mode.toggle();
//            if let Ok([camera_actor, mut parent_actor]) =
//                actor_query.get_many_mut([entity, parent.parent])
//            {
//                match camera.mode {
//                    CameraMode::FirstPerson => parent_actor.set_owner(Some(&camera_actor)),
//                    CameraMode::ThirdPerson => parent_actor.set_owner(None),
//                };
//            }
//
//            if let Ok((transform, sound)) = sound.get(parent.parent) {
//                play_sound_at_location(
//                    sound.camera_toggle,
//                    transform.position,
//                    transform.rotation,
//                    &ffi::SoundSettings::default(),
//                )
//            }
//        }
//    }
//}
//fn rotate_camera(mut query: Query<(&mut TransformComponent, &mut CameraComponent)>) {
//    fn lerp(start: f32, end: f32, t: f32) -> f32 {
//        start * (1.0 - t) + end * t
//    }
//    let mut x = 0.0;
//    let mut y = 0.0;
//
//    let max_angle = 85.0f32.to_radians();
//    unsafe {
//        (bindings().get_mouse_delta)(&mut x, &mut y);
//    }
//
//    for (mut spatial, mut cam) in query.iter_mut() {
//        let speed = 0.05;
//        cam.x += x * speed;
//        cam.y = f32::clamp(cam.y + y * speed, -max_angle, max_angle);
//
//        let smooth = 0.4;
//        cam.current_x = lerp(cam.current_x, cam.x, smooth);
//        cam.current_y = lerp(cam.current_y, cam.y, smooth);
//
//        spatial.rotation =
//            Quat::from_rotation_z(cam.current_x) * Quat::from_rotation_y(-cam.current_y);
//    }
//}
//
//
fn rotate_camera(input: Res<Input>, mut topdown: Query<&mut TopdownCameraComponent>, mut character: Query<&mut TransformComponent>) {
    if input.is_action_pressed(PlayerInput::ROTATE_CAMERA) {
        for mut cam in &mut topdown {
            let (x, _) = input.get_mouse_delta();
            cam.rotation += x * 0.03;
            

            if let Ok(mut transform) = character.get_mut(cam.target) {
                
                transform.rotation = Quat::from_rotation_z(cam.rotation);
                
            }
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
            topdown.target_position = Vec3::lerp(
                topdown.target_position,
                target_trans.position,
                    0.2,
            );
            camera_trans.position = topdown.target_position + camera_position;
            camera_trans.rotation = camera_rotation;
        }
    }
}

fn spawn_camera(mut commands: Commands, mut query: Query<(Entity, &TransformComponent, Added<PlayerComponent>)>) {
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
            => module
        };

        
        module
            .add_plugin(MovementPlugin)
            .add_startup_system_set(SystemSet::new().with_system(register_player_input))
            .add_system_set_to_stage(
                CoreStage::Update,
                SystemSet::new()
                    .with_system(create_player)
                    .with_system(spawn_camera)
                    .with_system(rotate_camera)
                    .with_system(update_camera.after(rotate_camera))
                    .with_system(update_controller_view.after(rotate_camera)),
            );
    }
}

unreal_api::implement_unreal_module!(MyModule);
