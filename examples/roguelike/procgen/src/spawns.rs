use crate::hull::HullCell;
use direction::CardinalDirection;
use grid_2d::{Coord, Grid};
use rand::{seq::SliceRandom, Rng};
use std::collections::{HashSet, VecDeque};

fn all_room_means(grid: &Grid<HullCell>) -> Vec<Coord> {
    let mut means = Vec::new();
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
                means.push(mean);
            }
        }
    }
    means
}

pub fn choose_player_spawn<R: Rng>(grid: &Grid<HullCell>, rng: &mut R) -> Coord {
    *all_room_means(grid).choose(rng).unwrap()
}
