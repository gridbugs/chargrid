#[derive(Clone, Copy)]
pub struct AudioProperties {
    pub volume: f32,
}

impl Default for AudioProperties {
    fn default() -> Self {
        Self { volume: 1. }
    }
}

impl AudioProperties {
    pub fn with_volume(self, volume: f32) -> Self {
        Self { volume, ..self }
    }
}

pub trait AudioPlayer {
    type Sound;
    fn play(&self, sound: &Self::Sound, properties: AudioProperties);
    fn load_sound(&self, bytes: &'static [u8]) -> Self::Sound;
}
