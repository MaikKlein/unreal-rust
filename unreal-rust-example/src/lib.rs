use std::fmt::format;

use unreal_api::{
    ecs::prelude::*,
    log,
    math::{Quat, Vec3},
    ffi,
    module::{bindings, UnrealModule},
};

pub struct ActorComponent {
    actor: *mut unreal_api::ffi::AActorOpaque,
}

unsafe impl Sync for ActorComponent {}
unsafe impl Send for ActorComponent {}
pub struct SpatialComponent {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

fn download_spatial_from_unreal(mut query: Query<(&ActorComponent, &mut SpatialComponent)>) {
    for (actor, mut spatial) in query.iter_mut() {
        let mut position = ffi::Vector3::default();
        let mut rotation = ffi::Quaternion::default();
        let mut scale = ffi::Vector3::default();

        (bindings().get_spatial_data)(
            actor.actor,
            &mut position,
            &mut rotation,
            &mut scale,
        );

        spatial.position = position.into();
        spatial.rotation = rotation.into();
        spatial.scale = scale.into();
    }
}
fn upload_spatial_to_unreal(mut query: Query<(&ActorComponent, &SpatialComponent)>) {
    for (actor, spatial) in query.iter() {
        (bindings().set_spatial_data)(
            actor.actor,
            spatial.position.into(),
            spatial.rotation.into(),
            spatial.scale.into(),
        );
    }
}

pub struct MyModule {
    world: World,
    schedule: Schedule,
}

pub static FOO: u32 = 444444;
impl UnrealModule for MyModule {
    fn initialize() -> Self {
        let mut schedule = Schedule::default();

        Self {
            world: World::new(),
            schedule,
        }
    }

    fn begin_play(&mut self) {
        log(bindings(), format!("Barrrr"));
    }

    fn tick(&mut self, dt: f32) {}
}
unreal_api::implement_unreal_module!(MyModule);
