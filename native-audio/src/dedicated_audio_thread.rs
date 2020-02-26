// Run audio on a separate thread, to prevent issue on windows
// when also using winit to create windows.
// More info: https://github.com/RustAudio/cpal/pull/348

use crate::{output_device, play_bytes_background, Error};
use prototty_audio::AudioProperties;
use std::sync::mpsc;
use std::thread;

pub struct NativeAudioPlayer {
    _audio_thread: thread::JoinHandle<()>,
    sender: mpsc::Sender<(&'static [u8], AudioProperties)>,
}

impl NativeAudioPlayer {
    pub fn try_new_default_device() -> Result<Self, Error> {
        let (sender, receiver): (_, mpsc::Receiver<(&'static [u8], AudioProperties)>) = mpsc::channel();
        let (init_sender, init_receiver) = mpsc::channel();
        let _audio_thread = std::thread::spawn(move || {
            if let Some(device) = output_device() {
                init_sender.send(Ok(())).unwrap();
                for (bytes, properties) in receiver {
                    play_bytes_background(&device, bytes, properties);
                }
                log::info!("dedicated audio thread stopped");
            } else {
                init_sender.send(Err(Error::NoOutputDevice)).unwrap();
            }
        });
        let () = init_receiver
            .recv()
            .expect("unable to get status of dedicated audio thread")?;
        Ok(Self { _audio_thread, sender })
    }

    pub fn new_default_device() -> Self {
        Self::try_new_default_device().unwrap()
    }

    pub fn play_bytes(&self, bytes: &'static [u8], properties: AudioProperties) {
        let result = self.sender.send((bytes, properties));
        match result {
            Ok(()) => (),
            Err(_) => log::error!("can't play audio because dedicated audio thread has stopped"),
        }
    }
}
