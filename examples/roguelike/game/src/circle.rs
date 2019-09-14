use grid_2d::Coord;
use static_circle::circle_with_squared_radius;

circle_with_squared_radius!(2112, RAW_COORDS, NUM_COORDS, Coord);

const COORDS: [Coord; 256] = RAW_COORDS;

const CARDINAL_RADIUS: i32 = 45;

pub fn coord(i: u8) -> Coord {
    COORDS[i as usize]
}

pub fn scale_to_cardinal_length(coord: Coord, length: i32) -> Coord {
    (coord * length) / CARDINAL_RADIUS
}
