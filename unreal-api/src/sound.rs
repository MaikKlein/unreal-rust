use glam::{Quat, Vec3};
use unreal_reflect::registry::USound;

pub use crate::ffi::SoundSettings;
use crate::module::bindings;

pub fn play_sound_at_location(
    sound: USound,
    location: Vec3,
    rotation: Quat,
    settings: &SoundSettings,
) {
    unsafe {
        (bindings().sound_fns.play_sound_at_location)(
            sound.ptr,
            location.into(),
            rotation.into(),
            settings,
        );
    }
}
