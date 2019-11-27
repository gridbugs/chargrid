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

impl<A: AudioPlayer> AudioPlayer for Option<A> {
    type Sound = Option<A::Sound>;
    fn play(&self, sound: &Self::Sound, properties: AudioProperties) {
        if let Some(player) = self.as_ref() {
            player.play(sound.as_ref().unwrap(), properties);
        }
    }
    fn load_sound(&self, bytes: &'static [u8]) -> Self::Sound {
        self.as_ref().map(|audio_player| audio_player.load_sound(bytes))
    }
}
