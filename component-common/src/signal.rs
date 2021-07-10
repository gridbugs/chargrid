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

pub struct SmoothSquareWave {
    constant: Duration,
    transition: Duration,
}

impl SmoothSquareWave {
    pub fn new(constant: Duration, transition: Duration) -> Self {
        Self {
            constant,
            transition,
        }
    }
}

impl SignalU8 for SmoothSquareWave {
    fn eval(&self, after: Duration) -> u8 {
        let constant = self.constant.as_millis();
        let transition = self.transition.as_millis();
        let mut remain_within_cycle = after.as_millis() % (2 * (constant + transition));
        if remain_within_cycle < constant {
            return 0;
        }
        remain_within_cycle -= constant;
        if remain_within_cycle < transition {
            return ((remain_within_cycle * 255) / transition) as u8;
        }
        remain_within_cycle -= transition;
        if remain_within_cycle < constant {
            return 255;
        }
        remain_within_cycle -= constant;
        return 255 - ((remain_within_cycle * 255) / transition) as u8;
    }
}
