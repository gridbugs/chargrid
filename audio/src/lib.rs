pub trait AudioPlayer {
    type Sound;
    fn play(&self, sound: &Self::Sound);
    fn load_sound(&self, bytes: &'static [u8]) -> Self::Sound;
}
