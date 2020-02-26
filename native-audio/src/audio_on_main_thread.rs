use crate::{output_device, play_bytes_background, Error};
use prototty_audio::AudioProperties;
use rodio::Device;

pub struct NativeAudioPlayer {
    device: Device,
}

impl NativeAudioPlayer {
    pub fn try_new_default_device() -> Result<Self, Error> {
        let device = output_device().ok_or(Error::NoOutputDevice)?;
        Ok(Self { device })
    }

    pub fn new_default_device() -> Self {
        Self::try_new_default_device().unwrap()
    }

    pub fn play_bytes(&self, bytes: &'static [u8], properties: AudioProperties) {
        play_bytes_background(&self.device, bytes, properties);
    }
}
