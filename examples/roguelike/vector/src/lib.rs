pub use coord_2d::Coord;
use rand::{
    distributions::uniform::{SampleBorrow, SampleUniform, UniformFloat, UniformSampler},
    Rng,
};
use rand_range::UniformLeftInclusiveRange;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct Cartesian {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct Radial {
    pub angle: Radians,
    pub length: f64,
}

impl Cartesian {
    pub fn from_coord(coord: Coord) -> Self {
        Self {
            x: coord.x as f64,
            y: coord.y as f64,
        }
    }
    pub fn to_coord_round_nearest(self) -> Coord {
        Coord::new(self.x.round() as i32, self.y.round() as i32)
    }
    pub fn to_radial(self) -> Radial {
        Radial {
            angle: Radians(self.y.atan2(self.x)),
            length: ((self.x * self.x) + (self.y * self.y)).sqrt(),
        }
    }
}

impl Radial {
    pub fn to_cartesian(self) -> Cartesian {
        Cartesian {
            x: self.length * self.angle.0.cos(),
            y: self.length * self.angle.0.sin(),
        }
    }
    pub fn rotate_clockwise(self, angle: Radians) -> Self {
        Self {
            angle: Radians(self.angle.0 + angle.0),
            length: self.length,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Radians(pub f64);

pub const PI: Radians = Radians(::std::f64::consts::PI);
pub const NEG_PI: Radians = Radians(-::std::f64::consts::PI);

impl Radians {
    pub const fn uniform_range_all() -> UniformLeftInclusiveRange<Self> {
        UniformLeftInclusiveRange { low: NEG_PI, high: PI }
    }
    pub fn random<R: Rng>(rng: &mut R) -> Self {
        Self(rng.gen_range(-::std::f64::consts::PI, ::std::f64::consts::PI))
    }
}

pub struct UniformRadians {
    inner: UniformFloat<f64>,
}

impl UniformSampler for UniformRadians {
    type X = Radians;
    fn new<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        Self {
            inner: UniformFloat::<f64>::new(low.borrow().0, high.borrow().0),
        }
    }
    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        UniformSampler::new(low, high)
    }
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        Radians(self.inner.sample(rng))
    }
}

impl SampleUniform for Radians {
    type Sampler = UniformRadians;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn conversion() {
        let cartesian = Cartesian { x: 42., y: 3. };
        let radial = cartesian.to_radial();
        let rotated_radial = radial.rotate_clockwise(Radians(::std::f64::consts::FRAC_PI_2));
        let rotated_cartesian = rotated_radial.to_cartesian();
        assert!((rotated_cartesian.x + cartesian.y) < 0.1);
        assert!((rotated_cartesian.y - cartesian.x) < 0.1);
    }
}
