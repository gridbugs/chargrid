// Run audio on a separate thread, to prevent issue on windows
// when also using winit to create windows.
// More info: https://github.com/RustAudio/cpal/pull/348

use crate::{output_device, play_bytes, play_bytes_loop, Error};
use rodio::Sink;
use std::sync::mpsc;
use std::thread;

struct PlayBytes {
    bytes: &'static [u8],
    loop_forever: bool,
}

type Request = PlayBytes;
type Response = Sink;

pub struct NativeAudioPlayer {
    _audio_thread: thread::JoinHandle<()>,
    send_request: mpsc::Sender<Request>,
    recv_response: mpsc::Receiver<Response>,
}

impl NativeAudioPlayer {
    pub fn try_new_default_device() -> Result<Self, Error> {
        let (send_request, recv_request) = mpsc::channel();
        let (send_response, recv_response) = mpsc::channel();
        let (send_init, recv_init) = mpsc::channel();
        let _audio_thread = thread::spawn(move || {
            if let Some(device) = output_device() {
                send_init.send(Ok(())).unwrap();
                for PlayBytes { bytes, loop_forever } in recv_request {
                    let handle = if loop_forever {
                        play_bytes_loop(&device, bytes)
                    } else {
                        play_bytes(&device, bytes)
                    };
                    send_response.send(handle).expect("failed to send response");
                }
                log::info!("dedicated audio thread stopped");
            } else {
                send_init.send(Err(Error::NoOutputDevice)).unwrap();
            }
        });
        let () = recv_init
            .recv()
            .expect("unable to get status of dedicated audio thread")?;
        Ok(Self {
            _audio_thread,
            send_request,
            recv_response,
        })
    }

    pub fn new_default_device() -> Self {
        Self::try_new_default_device().unwrap()
    }

    pub fn play_bytes(&self, bytes: &'static [u8]) -> Sink {
        self.play_bytes_gerneral(bytes, false)
    }

    pub fn play_bytes_loop(&self, bytes: &'static [u8]) -> Sink {
        self.play_bytes_gerneral(bytes, true)
    }

    fn play_bytes_gerneral(&self, bytes: &'static [u8], loop_forever: bool) -> Sink {
        match self.send_request.send(PlayBytes { bytes, loop_forever }) {
            Err(_) => {
                log::error!("can't play audio because dedicated audio thread has stopped");
                Sink::new_idle().0
            }
            Ok(()) => match self.recv_response.recv() {
                Err(_) => {
                    log::error!("no response from dedicated audio thread");
                    Sink::new_idle().0
                }
                Ok(sink) => sink,
            },
        }
    }
}
