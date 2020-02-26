use prototty_audio::{AudioPlayer, AudioProperties};
use rodio::{Decoder, Device, DeviceTrait, Sink, Source};
use std::io::Cursor;

pub(crate) fn output_device() -> Option<Device> {
    let maybe_device = rodio::default_output_device();
    if let Some(device) = maybe_device.as_ref() {
        match device.name() {
            Ok(device_name) => log::info!("Using audio device: {}", device_name),
            Err(error) => log::warn!("Using audio device with unknown name (error: {})", error),
        }
    } else {
        log::warn!("Unable to find audio device. Audio will be disabled.");
    }
    maybe_device
}

pub(crate) fn play_bytes_background(device: &Device, bytes: &'static [u8], properties: AudioProperties) {
    let sink = Sink::new(device);
    let source = Decoder::new(Cursor::new(bytes)).unwrap().amplify(properties.volume);
    sink.append(source);
    sink.detach();
}

#[derive(Debug)]
pub enum Error {
    NoOutputDevice,
}

#[cfg(not(any(target_os = "windows", feature = "force_dedicated_audio_thread")))]
mod audio_on_main_thread;
#[cfg(not(any(target_os = "windows", feature = "force_dedicated_audio_thread")))]
use audio_on_main_thread as platform_specific;

#[cfg(any(target_os = "windows", feature = "force_dedicated_audio_thread"))]
mod dedicated_audio_thread;
#[cfg(any(target_os = "windows", feature = "force_dedicated_audio_thread"))]
use dedicated_audio_thread as platform_specific;

pub use platform_specific::NativeAudioPlayer;

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
        NativeAudioPlayer::play_bytes(self, sound.bytes, properties)
    }
    fn load_sound(&self, bytes: &'static [u8]) -> Self::Sound {
        NativeSound::new(bytes)
    }
}
