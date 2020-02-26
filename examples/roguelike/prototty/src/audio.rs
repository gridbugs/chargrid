use maplit::hashmap;
use prototty_audio::AudioPlayer;
use std::collections::HashMap;

const EXPLOSION: &[u8] = include_bytes!("./audio/explosion.ogg");
const FIBERITRON: &[u8] = include_bytes!("./audio/fiberitron-loop.ogg");

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum Audio {
    Explosion,
    Fiberitron,
}

pub struct AudioTable<A: AudioPlayer> {
    map: HashMap<Audio, A::Sound>,
}

impl<A: AudioPlayer> AudioTable<A> {
    pub fn new(audio_player: &A) -> Self {
        let map = hashmap![
            Audio::Explosion => audio_player.load_sound(EXPLOSION),
            Audio::Fiberitron => audio_player.load_sound(FIBERITRON),
        ];
        Self { map }
    }
    pub fn get(&self, audio: Audio) -> &A::Sound {
        self.map.get(&audio).unwrap()
    }
}
