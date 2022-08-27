use crate::{USoundBaseOpague, Quaternion, Vector3};

#[repr(C)]
pub struct SoundSettings {
    pub volume: f32,
    pub pitch: f32,
}
impl Default for SoundSettings {
    fn default() -> Self {
        Self {
            volume: 1.0,
            pitch: 1.0,
        }
    }
}

extern "C" {
    pub fn PlaySoundAtLocation(
        sound: *const USoundBaseOpague,
        location: Vector3,
        rotation: Quaternion,
        settings: *const SoundSettings,
    );
}
pub type PlaySoundAtLocationFn = unsafe extern "C" fn(
    sound: *const USoundBaseOpague,
    location: Vector3,
    rotation: Quaternion,
    settings: *const SoundSettings,
);

#[repr(C)]
pub struct SoundFns {
    pub play_sound_at_location: PlaySoundAtLocationFn,
}
