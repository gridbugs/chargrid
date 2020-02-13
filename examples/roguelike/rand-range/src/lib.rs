use rand::{
    distributions::{
        uniform::{SampleUniform, Uniform},
        Distribution,
    },
    Rng,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct UniformInclusiveRange<T> {
    pub low: T,
    pub high: T,
}

impl<T: SampleUniform> UniformInclusiveRange<T> {
    pub fn choose<R: Rng>(&self, rng: &mut R) -> T {
        Uniform::<T>::new_inclusive(&self.low, &self.high).sample(rng)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct UniformLeftInclusiveRange<T> {
    pub low: T,
    pub high: T,
}

impl<T: SampleUniform> UniformLeftInclusiveRange<T> {
    pub fn choose<R: Rng>(&self, rng: &mut R) -> T {
        Uniform::<T>::new(&self.low, &self.high).sample(rng)
    }
}
