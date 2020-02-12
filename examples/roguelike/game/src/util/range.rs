use rand::Rng;
use rgb24::Rgb24;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurationRange {
    pub min: Duration,
    pub max: Duration,
}

impl DurationRange {
    pub fn choose<R: Rng>(&self, rng: &mut R) -> Duration {
        rng.gen_range(self.min, self.max)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rgb24Range {
    pub min: Rgb24,
    pub max: Rgb24,
}

impl Rgb24Range {
    pub fn choose<R: Rng>(self, rng: &mut R) -> Rgb24 {
        self.min.linear_interpolate(self.max, rng.gen())
    }
}
