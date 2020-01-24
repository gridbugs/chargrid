use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rational {
    pub numerator: u32,
    pub denominator: u32,
}

impl Rational {
    pub fn roll<R: Rng>(self, rng: &mut R) -> bool {
        rng.gen_range(0, self.denominator) < self.numerator
    }
}
