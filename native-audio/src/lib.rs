use prototty_audio::{AudioPlayer, AudioProperties};
use rodio::source::Source;
use rodio::{Decoder, Device, Sink};
use std::io::Cursor;

pub struct NativeAudioPlayer {
    device: Device,
}

#[derive(Debug)]
pub enum Error {
    NoOutputDevice,
    PanicDuringInit(Box<dyn std::any::Any + Send + 'static>),
}

impl NativeAudioPlayer {
    pub fn try_new_default_device() -> Result<Self, Error> {
        // Initialise audio on a separate thread, to prevent issue on windows
        // when also using winit to create windows.
        // More info: https://github.com/RustAudio/cpal/pull/348
        let init_thread = std::thread::spawn(|| rodio::default_output_device());
        match init_thread.join() {
            Ok(Some(device)) => Ok(Self { device }),
            Ok(None) => Err(Error::NoOutputDevice),
            Err(e) => Err(Error::PanicDuringInit(e)),
        }
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
