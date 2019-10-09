use direction::{CardinalDirections, OrdinalDirections};
use grid_2d::{Coord, Grid, Size};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GasRatio {
    numerator: u128,
    denominator: u128,
}

impl GasRatio {
    pub fn new(numerator: u128, denominator: u128) -> Self {
        if denominator == 0 {
            panic!("denominator may not be 0");
        }
        Self { numerator, denominator }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GasSpec {
    pub fade: GasRatio,
    pub spread: GasRatio,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GasCell {
    score: u64,
    next_score: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GasGrid {
    grid: Grid<GasCell>,
    count: u64,
    spec: GasSpec,
}

impl GasCell {
    fn log_score(&self) -> u8 {
        ((64 - self.score.leading_zeros()) * 4).min(255) as u8
    }
    fn new() -> Self {
        Self {
            score: 0,
            next_score: 0,
        }
    }
}

impl GasGrid {
    pub fn new(size: Size, spec: GasSpec) -> Self {
        let grid = Grid::new_fn(size, |_| GasCell::new());
        Self { grid, count: 0, spec }
    }
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    pub fn register(&mut self, coord: Coord) {
        if let Some(cell) = self.grid.get_mut(coord) {
            if cell.score == 0 {
                self.count += 1;
            }
            cell.score = u64::max_value();
        }
    }
    pub fn tick<F: Fn(Coord) -> bool>(&mut self, can_enter: F) {
        for dest_coord in self.grid.coord_iter() {
            if !can_enter(dest_coord) {
                continue;
            }
            let cardinal_sum: u128 = CardinalDirections
                .into_iter()
                .filter_map(|d| self.grid.get(dest_coord + d.coord()))
                .map(|c| c.score as u128)
                .sum();
            let ordinal_sum: u128 = OrdinalDirections
                .into_iter()
                .filter_map(|d| self.grid.get(dest_coord + d.coord()))
                .map(|c| c.score as u128)
                .sum();
            let weighted_average_neighbour_score = (cardinal_sum * 3 + ordinal_sum * 2) / (5 * 8);
            let to_add = (weighted_average_neighbour_score * self.spec.spread.numerator) / self.spec.spread.denominator;
            let dest_cell = self.grid.get_checked_mut(dest_coord);
            let score_before_adding = (dest_cell.score as u128
                * (self.spec.spread.denominator - self.spec.spread.numerator))
                / self.spec.spread.denominator;
            dest_cell.next_score = (((score_before_adding + to_add) * self.spec.fade.numerator)
                / self.spec.fade.denominator)
                .min(u64::max_value() as u128) as u64;
        }
        for cell in self.grid.iter_mut() {
            if cell.score == 0 && cell.next_score != 0 {
                self.count += 1;
            } else if cell.score != 0 && cell.next_score == 0 {
                self.count -= 1;
            }
            cell.score = cell.next_score;
        }
    }
    pub fn iter<'a>(&'a self) -> impl 'a + Iterator<Item = u8> {
        self.grid.iter().map(|cell| cell.log_score())
    }
}
