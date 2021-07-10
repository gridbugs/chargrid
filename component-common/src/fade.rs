use crate::signal::{Linear, SignalU8};
use chargrid_component::Rgba32;
use std::time::Duration;

pub struct Fade<S: SignalU8> {
    pub lo: Rgba32,
    pub hi: Rgba32,
    pub signal: S,
}

impl<S: SignalU8> Fade<S> {
    pub fn eval(&self, after: Duration) -> Rgba32 {
        self.lo.linear_interpolate(self.hi, self.signal.eval(after))
    }
}

pub fn linear(lo: Rgba32, hi: Rgba32, duration: Duration) -> Fade<Linear> {
    Fade {
        lo,
        hi,
        signal: Linear::with_duration(duration),
    }
}
