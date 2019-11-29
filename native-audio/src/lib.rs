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

pub use platform_specific::{NativeAudioPlayer, NativeSound};
