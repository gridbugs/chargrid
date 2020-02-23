use crate::hull::HullCell;
use direction::CardinalDirection;
use grid_2d::{Coord, Grid};
use rand::Rng;
use std::collections::{HashSet, VecDeque};

#[derive(Debug)]
pub enum LightType {
    Working,
    Flickering,
    Emergency,
    Broken,
}

impl LightType {
    fn choose<R: Rng>(rng: &mut R) -> Self {
        match rng.gen_range(0, 100) {
            0..=19 => Self::Broken,
            20..=24 => Self::Flickering,
            25..=34 => Self::Emergency,
            35..=99 => Self::Working,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct Light {
    pub coord: Coord,
    pub typ: LightType,
}

pub fn choose_lights<R: Rng>(grid: &Grid<HullCell>, rng: &mut R) -> Vec<Light> {
    let mut lights = Vec::new();
    let mut flood_fill_buffer = VecDeque::new();
    let mut visited = HashSet::new();
    for (coord, cell) in grid.enumerate() {
        if let HullCell::Floor = cell {
            if visited.insert(coord) {
                flood_fill_buffer.push_back(coord);
                let mut total = Coord::new(0, 0);
                let mut count = 0;
                while let Some(coord) = flood_fill_buffer.pop_front() {
                    total += coord;
                    count += 1;
                    for direction in CardinalDirection::all() {
                        let neighbour_coord = coord + direction.coord();
                        if let Some(HullCell::Floor) = grid.get(neighbour_coord) {
                            if visited.insert(neighbour_coord) {
                                flood_fill_buffer.push_back(neighbour_coord);
                            }
                        }
                    }
                }
                let mean = total / count;
                let typ = LightType::choose(rng);
                lights.push(Light { coord: mean, typ });
            }
        }
    }
    lights
}
