use std::{collections::HashMap, ffi::CStr};

use bevy_ecs::prelude::*;
use unreal_api::{
    ffi::{self, AActorOpaque, ActionState},
    iterate_actors,
    math::{Quat, Vec3},
    module::{bindings, UnrealModule},
};
#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
enum CoreStage {
    Startup,
    PreUpdate,
    Update,
    PostUpdate,
}
#[derive(Default, Debug, Copy, Clone)]
pub struct Frame {
    pub dt: f32,
}

pub struct ActorComponent {
    ptr: ActorPtr
}

#[derive(Default, Debug)]
pub struct SpatialComponent {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}
#[derive(Default, Debug)]
pub struct MovementComponent {
    pub velocity: Vec3,
}

// TODO: Implement unregister. Might need Removed<T> in bevy
#[derive(Default)]
pub struct ActorRegistration {
    pub actor_to_entity: HashMap<ActorPtr, Entity>,
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ActorPtr(pub *mut AActorOpaque);
unsafe impl Send for ActorPtr {}
unsafe impl Sync for ActorPtr {}

fn download_spatial_from_unreal(mut query: Query<(&ActorComponent, &mut SpatialComponent)>) {
    for (actor, mut spatial) in query.iter_mut() {
        let mut position = ffi::Vector3::default();
        let mut rotation = ffi::Quaternion::default();
        let mut scale = ffi::Vector3::default();

        (bindings().get_spatial_data)(actor.ptr.0, &mut position, &mut rotation, &mut scale);

        spatial.position = position.into();
        spatial.rotation = rotation.into();
        spatial.scale = scale.into();
    }
}
fn upload_spatial_to_unreal(query: Query<(&ActorComponent, &SpatialComponent)>) {
    for (actor, spatial) in query.iter() {
        (bindings().set_spatial_data)(
            actor.ptr.0,
            spatial.position.into(),
            spatial.rotation.into(),
            spatial.scale.into(),
        );
    }
}

fn compute_movement_velocity(mut query: Query<&mut MovementComponent>) {
    let forward = {
        let s = CStr::from_bytes_with_nul("MoveForward\0".as_bytes()).unwrap();
        let mut value = 0.0;
        (bindings().get_axis_value)(s.as_ptr(), &mut value);
        value
    };
    let right = {
        let s = CStr::from_bytes_with_nul("MoveRight\0".as_bytes()).unwrap();
        let mut value = 0.0;
        (bindings().get_axis_value)(s.as_ptr(), &mut value);
        value
    };

    for mut movement in query.iter_mut() {
        movement.velocity = Vec3::new(forward, right, 0.0).normalize_or_zero() * 500.0;
    }
}

fn update_velocity(
    frame: Res<Frame>,
    mut query: Query<(&mut SpatialComponent, &MovementComponent)>,
) {
    for (mut spatial, movement) in query.iter_mut() {
        spatial.position += movement.velocity * frame.dt;

        if movement.velocity.length() > 0.0 {
            let target_rot =
                Quat::from_rotation_z(f32::atan2(movement.velocity.y, movement.velocity.x));
            spatial.rotation = Quat::lerp(spatial.rotation, target_rot, frame.dt * 12.0);
        }
    }
}
fn rotate(frame: Res<Frame>, mut query: Query<&mut SpatialComponent>) {
    for mut spatial in query.iter_mut() {
        spatial.rotation = Quat::from_rotation_z(1.5 * frame.dt) * spatial.rotation;
    }
}

fn register_actors(mut actor_register: ResMut<ActorRegistration>, mut commands: Commands) {
    for actor in iterate_actors(bindings()) {
        let entity = commands
            .spawn()
            .insert_bundle((
                ActorComponent { ptr: ActorPtr(actor) },
                SpatialComponent::default(),
                MovementComponent::default(),
            ))
            .id();

        actor_register
            .actor_to_entity
            .insert(ActorPtr(actor), entity);

        //(bindings().set_entity_for_actor)(
        //    actor,
        //    ffi::Entity {
        //        id: entity.to_bits(),
        //    },
        //);
    }
    //log::info!("Registerd {} actors", actors.len());
}
fn set_entity_id(mut query: Query<(Entity, &mut ActorComponent)>) {
    for (entity, actor) in query.iter_mut() {
        (bindings().set_entity_for_actor)(
            actor.ptr.0,
            ffi::Entity {
                id: entity.to_bits(),
            },
        );
    }
}
pub struct MyModule {
    world: World,
    schedule: Schedule,
    startup: Schedule,
}

impl UnrealModule for MyModule {
    fn initialize() -> Self {
        log::info!("Initialize Rust");
        let mut startup = Schedule::default();
        startup.add_stage(CoreStage::Startup, SystemStage::single_threaded());

        let mut schedule = Schedule::default();
        schedule
            .add_stage(CoreStage::PreUpdate, SystemStage::single_threaded())
            .add_stage(CoreStage::Update, SystemStage::single_threaded())
            .add_stage(CoreStage::PostUpdate, SystemStage::single_threaded());

        schedule.add_system_to_stage(CoreStage::PreUpdate, download_spatial_from_unreal.system());
        //schedule.add_system_to_stage(CoreStage::Update, rotate.system());
        schedule.add_system_to_stage(CoreStage::Update, compute_movement_velocity.system());
        schedule.add_system_to_stage(CoreStage::Update, update_velocity.system());
        schedule.add_system_to_stage(CoreStage::PostUpdate, upload_spatial_to_unreal.system());

        Self {
            world: World::new(),
            schedule,
            startup,
        }
    }

    fn begin_play(&mut self) {
        *self = Self::initialize();
        log::info!("BeginPlay Rust");
        self.world.insert_resource(Frame::default());
        self.world.insert_resource(ActorRegistration::default());
        let mut startup = Schedule::default();
        startup.add_stage(CoreStage::Startup, SystemStage::single_threaded());
        startup.add_system_to_stage(CoreStage::Startup, register_actors.system());
        //startup.add_system_to_stage(CoreStage::Startup, set_entity_id.system());
        startup.run_once(&mut self.world);
        self.startup.run_once(&mut self.world);
    }

    fn tick(&mut self, dt: f32) {
        //let mut state = ActionState::Nothing;
        //let s = CStr::from_bytes_with_nul("Jump\0".as_bytes()).unwrap();
        //(bindings().get_action_state)(s.as_ptr(), &mut state);
        //log::info!("Jump {:?}", state);
        if let Some(mut frame) = self.world.get_resource_mut::<Frame>() {
            frame.dt = dt;
        }
        self.schedule.run_once(&mut self.world);
    }
}
unreal_api::implement_unreal_module!(MyModule);
