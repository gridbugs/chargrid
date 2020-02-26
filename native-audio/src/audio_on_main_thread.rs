use crate::{output_device, play_bytes, play_bytes_loop, Error};
use rodio::{Device, Sink};

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

    pub fn play_bytes(&self, bytes: &'static [u8]) -> Sink {
        play_bytes(&self.device, bytes)
    }

    pub fn play_bytes_loop(&self, bytes: &'static [u8]) -> Sink {
        play_bytes_loop(&self.device, bytes)
    }
}
