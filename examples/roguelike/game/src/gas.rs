use direction::{Direction, DirectionsCardinal, DirectionsOrdinal};
use grid_2d::{Coord, Grid, Size};
use serde::{Deserialize, Serialize};
use std::mem;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GasCell {
    score: u64,
}

impl GasCell {
    fn log_score(&self) -> u8 {
        ((64 - self.score.leading_zeros()) * 4).min(255) as u8
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GasGrid {
    current: Grid<GasCell>,
    next: Grid<GasCell>,
    count: u64,
}

struct NeighbourScores {
    total: u128,
    count: u8,
}

fn visit_neighbours<F, D>(grid: &Grid<GasCell>, coord: Coord, can_enter: F, directions: D) -> NeighbourScores
where
    F: Fn(Coord) -> bool,
    D: IntoIterator<Item = Direction>,
{
    let mut ret = NeighbourScores { total: 0, count: 0 };
    for direction in directions {
        let neighbour_coord = coord + direction.coord();
        if let Some(GasCell { score }) = grid.get(neighbour_coord) {
            if can_enter(neighbour_coord) {
                ret.count += 1;
                ret.total += *score as u128;
            }
        }
    }
    ret
}

fn compute_next_score(current_score: u128, _cardinal: NeighbourScores, _ordinal: NeighbourScores) -> u64 {
    (current_score / 8) as u64
}

impl GasGrid {
    pub fn new(size: Size) -> Self {
        let current = Grid::new_clone(size, GasCell { score: 0 });
        let next = current.clone();
        let count = 0;
        Self { current, next, count }
    }
    pub fn register(&mut self, coord: Coord) {
        let cell = self.current.get_checked_mut(coord);
        if cell.score == 0 {
            self.count += 1;
        }
        cell.score = u64::max_value();
    }
    pub fn tick<F>(&mut self, can_enter: F)
    where
        F: Fn(Coord) -> bool,
    {
        if self.count == 0 {
            return;
        }
        for coord in self.current.coord_iter() {
            let current_cell = self.current.get_checked(coord);
            let current_score = current_cell.score as u128;
            let cardinal = visit_neighbours(&self.current, coord, &can_enter, DirectionsCardinal);
            let ordinal = visit_neighbours(&self.current, coord, &can_enter, DirectionsOrdinal);
            let next_score = compute_next_score(current_score, cardinal, ordinal);
            if next_score == 0 && current_cell.score != 0 {
                self.count -= 1;
            } else if next_score != 0 && current_cell.score == 0 {
                self.count += 1;
            }
            let next_cell = self.next.get_checked_mut(coord);
            next_cell.score = next_score;
        }
        mem::swap(&mut self.current, &mut self.next);
    }
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    pub fn intensity(&self, coord: Coord) -> u8 {
        self.current.get_checked(coord).log_score()
    }
}
