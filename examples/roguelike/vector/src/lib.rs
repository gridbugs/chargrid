pub use coord_2d::Coord;

#[derive(Debug, Clone, Copy)]
pub struct Cartesian {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct Radial {
    pub angle_radians: f64,
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
            angle_radians: self.y.atan2(self.x),
            length: ((self.x * self.x) + (self.y * self.y)).sqrt(),
        }
    }
}

impl Radial {
    pub fn to_cartesian(self) -> Cartesian {
        Cartesian {
            x: self.length * self.angle_radians.cos(),
            y: self.length * self.angle_radians.sin(),
        }
    }
    pub fn rotate_clockwise(self, angle_radians: f64) -> Self {
        Self {
            angle_radians: self.angle_radians + angle_radians,
            length: self.length,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn conversion() {
        let cartesian = Cartesian { x: 42., y: 3. };
        let radial = cartesian.to_radial();
        let rotated_radial = radial.rotate_clockwise(::std::f64::consts::FRAC_PI_2);
        let rotated_cartesian = rotated_radial.to_cartesian();
        assert!((rotated_cartesian.x + cartesian.y) < 0.1);
        assert!((rotated_cartesian.y - cartesian.x) < 0.1);
    }
}
