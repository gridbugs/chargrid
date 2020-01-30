use crate::common;
use crate::Error;
use prototty_audio::{AudioPlayer, AudioProperties};
use rodio::source::Source;
use rodio::{Decoder, Device, Sink};
use std::io::Cursor;

pub struct NativeAudioPlayer {
    device: Device,
}
impl NativeAudioPlayer {
    pub fn try_new_default_device() -> Result<Self, Error> {
        let device = common::output_device().ok_or(Error::NoOutputDevice)?;
        Ok(Self { device })
    }

    pub fn new_default_device() -> Self {
        Self::try_new_default_device().unwrap()
    }

    pub fn play(&self, sound: &NativeSound, properties: AudioProperties) {
        let sink = Sink::new(&self.device);
        let source = Decoder::new(Cursor::new(sound.bytes))
            .unwrap()
            .amplify(properties.volume);
        sink.append(source);
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
    fn play(&self, sound: &Self::Sound, properties: AudioProperties) {
        NativeAudioPlayer::play(self, sound, properties)
    }
    fn load_sound(&self, bytes: &'static [u8]) -> Self::Sound {
        NativeSound::new(bytes)
    }
}
