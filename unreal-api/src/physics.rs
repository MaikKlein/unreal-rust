use ffi::AActorOpaque;
use glam::{Quat, Vec3};
use unreal_ffi as ffi;

use crate::{core::PhysicsComponent, module::bindings};

pub struct SweepResult {
    pub impact_location: Vec3,
    pub location: Vec3,
    pub penetration_depth: f32,
    pub normal: Vec3,
}

#[derive(Default)]
pub struct SweepParams {
    pub ignored_actors: Vec<*mut AActorOpaque>,
}
pub fn line_trace(
    start: Vec3,
    end: Vec3,
    params: SweepParams,
) -> Option<SweepResult> {
    let params = ffi::LineTraceParams {
        ignored_actors: params.ignored_actors.as_ptr(),
        ignored_actors_len: params.ignored_actors.len(),
    };
    let mut hit = ffi::HitResult::default();
    if (bindings().physics_bindings.line_trace)(
        start.into(),
        end.into(),
        params,
        &mut hit,
    ) == 1
    {
        Some(SweepResult {
            impact_location: hit.impact_location.into(),
            location: hit.location.into(),
            normal: hit.normal.into(),
            penetration_depth: hit.pentration_depth,
        })
    } else {
        None
    }
}


pub fn sweep(
    start: Vec3,
    end: Vec3,
    rotation: Quat,
    physics: &PhysicsComponent,
    params: SweepParams,
) -> Option<SweepResult> {
    let params = ffi::LineTraceParams {
        ignored_actors: params.ignored_actors.as_ptr(),
        ignored_actors_len: params.ignored_actors.len(),
    };
    let mut hit = ffi::HitResult::default();
    if (bindings().physics_bindings.sweep)(
        start.into(),
        end.into(),
        rotation.into(),
        params,
        physics.ptr.ptr,
        &mut hit,
    ) == 1
    {
        Some(SweepResult {
            impact_location: hit.impact_location.into(),
            location: hit.location.into(),
            normal: hit.normal.into(),
            penetration_depth: hit.pentration_depth,
        })
    } else {
        None
    }
}
