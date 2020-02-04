use hashbrown::HashMap;
use prototty_audio::AudioPlayer;

const EXPLOSION: &[u8] = include_bytes!("./audio/explosion.ogg");

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum Audio {
    Explosion,
}

pub struct AudioTable<A: AudioPlayer> {
    map: HashMap<Audio, A::Sound>,
}

impl<A: AudioPlayer> AudioTable<A> {
    pub fn new(audio_player: &A) -> Self {
        let mut map = HashMap::new();
        map.insert(Audio::Explosion, audio_player.load_sound(EXPLOSION));
        Self { map }
    }
    pub fn get(&self, audio: Audio) -> &A::Sound {
        self.map.get(&audio).unwrap()
    }
}
