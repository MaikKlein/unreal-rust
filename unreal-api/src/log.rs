use glam::{Quat, Vec3};
use log::{set_boxed_logger, set_max_level, LevelFilter, Metadata, Record, SetLoggerError};
use unreal_ffi as ffi;
use unreal_ffi::Color;

use crate::{
    core::{ActorPtr, CollisionShape},
    module::bindings,
};
struct UnrealLogger;

impl log::Log for UnrealLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        // TODO
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let text = record.args().to_string();
            (crate::module::bindings().log)(text.as_ptr() as *const _, text.len() as i32);
        }
    }

    fn flush(&self) {}
}

pub fn init() -> Result<(), SetLoggerError> {
    set_boxed_logger(Box::new(UnrealLogger)).map(|()| set_max_level(LevelFilter::Info))
}

pub fn visual_log_capsule(
    category: LogCategory,
    actor: ActorPtr,
    position: Vec3,
    rotation: Quat,
    half_height: f32,
    radius: f32,
    color: Color,
) {
    unsafe {
        (bindings().visual_log_capsule)(
            ffi::Utf8Str::from(category.name),
            actor.0,
            position.into(),
            rotation.into(),
            half_height,
            radius,
            color,
        );
    }
}

pub fn visual_log_shape(
    category: LogCategory,
    actor: ActorPtr,
    position: Vec3,
    rotation: Quat,
    shape: CollisionShape,
    color: Color,
) {
    match shape {
        CollisionShape::Capsule {
            half_height,
            radius,
        } => unsafe {
            (bindings().visual_log_capsule)(
                ffi::Utf8Str::from(category.name),
                actor.0,
                position.into(),
                rotation.into(),
                half_height,
                radius,
                color,
            );
        },
        CollisionShape::Box { half_extent: _ } => {
            unimplemented!()
        }
        CollisionShape::Sphere { radius } => unsafe {
            (bindings().visual_log_location)(
                ffi::Utf8Str::from(category.name),
                actor.0,
                position.into(),
                radius,
                color,
            );
        },
    }
}

pub fn visual_log_location(
    category: LogCategory,
    actor: ActorPtr,
    position: Vec3,
    radius: f32,
    color: Color,
) {
    unsafe {
        (bindings().visual_log_location)(
            ffi::Utf8Str::from(category.name),
            actor.0,
            position.into(),
            radius,
            color,
        );
    }
}

#[derive(Copy, Clone)]
pub struct LogCategory {
    pub name: &'static str,
}

impl LogCategory {
    pub const fn new(name: &'static str) -> Self {
        Self { name }
    }
}
