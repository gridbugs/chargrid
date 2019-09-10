use grid_2d::Coord;
use rand::Rng;
use static_circle::circle_with_squared_radius;

circle_with_squared_radius!(2112, RAW_COORDS, NUM_COORDS, Coord);

const COORDS: [Coord; 256] = RAW_COORDS;

const CARDINAL_RADIUS: i32 = 45;

fn coord(i: u8) -> Coord {
    COORDS[i as usize]
}

fn scale_to_cardinal_length(coord: Coord, length: i32) -> Coord {
    (coord * length) / CARDINAL_RADIUS
}

pub fn random_coord_with_cardinal_length<R: Rng>(length: i32, rng: &mut R) -> Coord {
    scale_to_cardinal_length(coord(rng.gen()), length)
}
