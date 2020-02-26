pub trait AudioPlayer {
    type Sound;
    type Handle: AudioHandle;
    #[must_use]
    fn play(&self, sound: &Self::Sound) -> Self::Handle;
    #[must_use]
    fn play_loop(&self, sound: &Self::Sound) -> Self::Handle;
    fn load_sound(&self, bytes: &'static [u8]) -> Self::Sound;
}

pub trait AudioHandle {
    fn set_volume(&self, volume: f32);
    fn background(self);
}

impl<A: AudioPlayer> AudioPlayer for Option<A> {
    type Sound = Option<A::Sound>;
    type Handle = Option<A::Handle>;
    fn play(&self, sound: &Self::Sound) -> Self::Handle {
        self.as_ref().and_then(|a| sound.as_ref().map(|s| a.play(s)))
    }
    fn play_loop(&self, sound: &Self::Sound) -> Self::Handle {
        self.as_ref().and_then(|a| sound.as_ref().map(|s| a.play_loop(s)))
    }
    fn load_sound(&self, bytes: &'static [u8]) -> Self::Sound {
        self.as_ref().map(|a| a.load_sound(bytes))
    }
}

impl<H: AudioHandle> AudioHandle for Option<H> {
    fn set_volume(&self, volume: f32) {
        if let Some(handle) = self.as_ref() {
            handle.set_volume(volume);
        }
    }
    fn background(self) {
        if let Some(handle) = self {
            handle.background()
        }
    }
}
