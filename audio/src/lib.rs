#[derive(Clone, Copy, Debug)]
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

impl<A: AudioPlayer> AudioPlayer for Option<A> {
    type Sound = Option<A::Sound>;
    fn play(&self, sound: &Self::Sound, properties: AudioProperties) {
        match (self.as_ref(), sound.as_ref()) {
            (Some(a), Some(s)) => a.play(s, properties),
            _ => (),
        }
    }
    fn load_sound(&self, bytes: &'static [u8]) -> Self::Sound {
        self.as_ref().map(|a| a.load_sound(bytes))
    }
}
