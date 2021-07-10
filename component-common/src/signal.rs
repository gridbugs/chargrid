use std::time::Duration;

pub trait SignalU8: Sized {
    fn eval(&self, after: Duration) -> u8;

    fn min(self, min: u8) -> Min<Self> {
        Min { signal: self, min }
    }
}

pub struct Linear {
    duration: Duration,
}

impl Linear {
    pub fn with_duration(duration: Duration) -> Self {
        Self { duration }
    }
    pub fn with_step_duration(step_duration: Duration) -> Self {
        Self::with_duration(step_duration * 255)
    }
}

impl SignalU8 for Linear {
    fn eval(&self, after: Duration) -> u8 {
        if after >= self.duration {
            255
        } else {
            (after.as_millis() * 255u128 / self.duration.as_millis()) as u8
        }
    }
}

pub struct SquareWave01 {
    half_period: Duration,
}

impl SquareWave01 {
    pub fn with_half_period(half_period: Duration) -> Self {
        Self { half_period }
    }

    pub fn eval_bool(&self, after: Duration) -> bool {
        (after.as_millis() / self.half_period.as_millis()) % 2 == 0
    }
}

impl SignalU8 for SquareWave01 {
    fn eval(&self, after: Duration) -> u8 {
        self.eval_bool(after) as u8
    }
}

pub struct Min<S: SignalU8> {
    pub signal: S,
    pub min: u8,
}

impl<S: SignalU8> SignalU8 for Min<S> {
    fn eval(&self, after: Duration) -> u8 {
        self.signal.eval(after).min(self.min)
    }
}
