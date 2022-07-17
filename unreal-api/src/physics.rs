use ffi::AActorOpaque;
use glam::{Quat, Vec3};
use unreal_ffi as ffi;

use crate::{core::CollisionShape, module::bindings};

#[derive(Debug)]
pub struct SweepResult {
    pub impact_location: Vec3,
    pub location: Vec3,
    pub penetration_depth: f32,
    pub normal: Vec3,
    pub impact_normal: Vec3,
    pub start_in_penentration: bool,
}

#[derive(Clone, Default)]
pub struct SweepParams {
    pub ignored_actors: Vec<*mut AActorOpaque>,
}
pub fn line_trace(start: Vec3, end: Vec3, params: SweepParams) -> Option<SweepResult> {
    let params = ffi::LineTraceParams {
        ignored_actors: params.ignored_actors.as_ptr(),
        ignored_actors_len: params.ignored_actors.len(),
    };
    let mut hit = ffi::HitResult::default();
    unsafe {
        if (bindings().physics_bindings.line_trace)(start.into(), end.into(), params, &mut hit) == 1
        {
            Some(SweepResult {
                impact_location: hit.impact_location.into(),
                location: hit.location.into(),
                normal: hit.normal.into(),
                penetration_depth: hit.pentration_depth,
                start_in_penentration: hit.start_penetrating == 1,
                impact_normal: hit.impact_normal.into(),
            })
        } else {
            None
        }
    }
}

pub fn sweep(
    start: Vec3,
    end: Vec3,
    rotation: Quat,
    collision_shape: CollisionShape,
    params: SweepParams,
) -> Option<SweepResult> {
    let params = ffi::LineTraceParams {
        ignored_actors: params.ignored_actors.as_ptr(),
        ignored_actors_len: params.ignored_actors.len(),
    };
    let mut hit = ffi::HitResult::default();
    unsafe {
        if (bindings().physics_bindings.sweep)(
            start.into(),
            end.into(),
            rotation.into(),
            params,
            collision_shape.into(),
            &mut hit,
        ) == 1
        {
            Some(SweepResult {
                impact_location: hit.impact_location.into(),
                location: hit.location.into(),
                normal: hit.normal.into(),
                penetration_depth: hit.pentration_depth,
                start_in_penentration: hit.start_penetrating == 1,
                impact_normal: hit.impact_normal.into(),
            })
        } else {
            None
        }
    }
}
pub fn sweep_multi(
    start: Vec3,
    end: Vec3,
    rotation: Quat,
    collision_shape: CollisionShape,
    max_results: usize,
    params: SweepParams,
) -> Option<Vec<SweepResult>> {
    let params = ffi::LineTraceParams {
        ignored_actors: params.ignored_actors.as_ptr(),
        ignored_actors_len: params.ignored_actors.len(),
    };
    let mut hits: Vec<ffi::HitResult> = Vec::new();
    hits.resize_with(max_results, Default::default);
    unsafe {
        let len = (bindings().physics_bindings.sweep_multi)(
            start.into(),
            end.into(),
            rotation.into(),
            params,
            collision_shape.into(),
            max_results,
            hits.as_mut_ptr(),
        );
        hits.truncate(len as usize);
        if len > 0 {
            Some(
                hits.into_iter()
                    .map(|hit| SweepResult {
                        impact_location: hit.impact_location.into(),
                        location: hit.location.into(),
                        normal: hit.normal.into(),
                        penetration_depth: hit.pentration_depth,
                        start_in_penentration: hit.start_penetrating == 1,
                        impact_normal: hit.impact_normal.into(),
                    })
                    .collect(),
            )
        } else {
            None
        }
    }
}
