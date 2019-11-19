use prototty_audio::AudioPlayer;
use rodio::{Decoder, Device, Sink};
use std::io::Cursor;

pub struct NativeAudioPlayer {
    device: Device,
}

impl NativeAudioPlayer {
    pub fn try_new_default_device() -> Option<Self> {
        let device = rodio::default_output_device()?;
        Some(Self { device })
    }

    pub fn new_default_device() -> Self {
        Self::try_new_default_device().unwrap()
    }

    pub fn play(&self, sound: &NativeSound) {
        let sink = Sink::new(&self.device);
        sink.append(Decoder::new(Cursor::new(sound.bytes)).unwrap());
        sink.detach();
    }
}

#[derive(Clone)]
pub struct NativeSound {
    bytes: &'static [u8],
}

impl NativeSound {
    pub fn new(bytes: &'static [u8]) -> Self {
        Self { bytes }
    }
}

impl AudioPlayer for NativeAudioPlayer {
    type Sound = NativeSound;
    fn play(&self, sound: &Self::Sound) {
        NativeAudioPlayer::play(self, sound)
    }
    fn load_sound(&self, bytes: &'static [u8]) -> Self::Sound {
        NativeSound::new(bytes)
    }
}
