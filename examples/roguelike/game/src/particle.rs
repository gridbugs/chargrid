use crate::data::{GameData, Id};
use direction::*;
use grid_2d::{Grid, Size};
use line_2d::{Coord, LineSegment, NodeIter};
use rgb24::Rgb24;
use serde::{Deserialize, Serialize};
use std::mem;
use std::time::Duration;

const FRAME_DURATION: Duration = Duration::from_micros(1_000_000 / 60);

#[derive(Serialize, Deserialize)]
pub struct Particle {
    cardinal_step_duration: Duration,
    ordinal_step_duration: Duration,
    until_next_step: Duration,
    path_iter: NodeIter,
    path: LineSegment,
    entity_id: Id,
}

#[derive(Serialize, Deserialize)]
pub struct TrailCell {
    score: u64,
    next_score: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Trails {
    grid: Grid<TrailCell>,
    count: u64,
}

#[derive(Serialize, Deserialize)]
pub struct ParticleSystem {
    active_particles: Vec<Particle>,
    next_active_particles: Vec<Particle>,
    trails: Trails,
}

enum ControlFlow {
    Continue,
    Break,
}

impl Particle {
    fn ordinal_duration_from_cardinal_duration(duration: Duration) -> Duration {
        const SQRT_2_X_1_000_000: u128 = 1414214;
        let micros = duration.as_micros();
        let diagonal_micros = (micros * SQRT_2_X_1_000_000) / 1_000_000;
        Duration::from_micros(diagonal_micros as u64)
    }
    pub fn new(path: LineSegment, step_duration: Duration, entity_id: Id) -> Self {
        Self {
            cardinal_step_duration: step_duration,
            ordinal_step_duration: Self::ordinal_duration_from_cardinal_duration(step_duration),
            until_next_step: Duration::from_millis(0),
            path_iter: path.node_iter(),
            path,
            entity_id,
        }
    }
    fn tick(&mut self, trails: &mut Trails, game_data: &mut GameData) -> ControlFlow {
        let since_last_tick = FRAME_DURATION;
        if since_last_tick < self.until_next_step {
            self.until_next_step -= since_last_tick;
            ControlFlow::Continue
        } else {
            let mut timeslice = since_last_tick - self.until_next_step;
            loop {
                let current_coord = self.path_iter.current();
                if let Some(node) = self.path_iter.next() {
                    let step_duration = if node.prev.is_cardinal() {
                        self.cardinal_step_duration
                    } else {
                        self.ordinal_step_duration
                    };
                    if node.coord != self.path.start() {
                        trails.register(current_coord);
                    }
                    if let Some(cell) = game_data.get_cell(node.coord) {
                        if cell.is_solid() {
                            break ControlFlow::Break;
                        }
                    } else {
                        break ControlFlow::Break;
                    }
                    if let Some(remaining_timeslice) = timeslice.checked_sub(step_duration) {
                        timeslice = remaining_timeslice;
                    } else {
                        self.until_next_step = step_duration - timeslice;
                        break ControlFlow::Continue;
                    }
                } else {
                    trails.register(current_coord);
                    break ControlFlow::Break;
                }
            }
        }
    }
    pub fn coord(&self) -> Coord {
        self.path_iter.current()
    }
}

impl TrailCell {
    pub fn log_score(&self) -> u8 {
        ((64 - self.score.leading_zeros()) * 4).min(255) as u8
    }
    pub fn col(&self) -> Option<Rgb24> {
        if self.score == 0 {
            None
        } else {
            let s = self.log_score();
            Some(Rgb24::new(0, 0, 0).linear_interpolate(Rgb24::new(255, 255, 255), s))
        }
    }
    fn new() -> Self {
        Self {
            score: 0,
            next_score: 0,
        }
    }
}

impl Trails {
    fn register(&mut self, coord: Coord) {
        if let Some(cell) = self.grid.get_mut(coord) {
            if cell.score == 0 {
                self.count += 1;
            }
            cell.score = u64::max_value();
        }
    }
    fn spread(&mut self, game_data: &GameData) {
        for (dest_coord, game_cell) in self.grid.coord_iter().zip(game_data.grid().iter()) {
            if game_cell.is_solid() {
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
            const FADE_RATIO: (u128, u128) = (1, 10);
            const SPREAD_RATIO: (u128, u128) = (1, 100);
            const REMAIN_RATIO: (u128, u128) = (SPREAD_RATIO.1 - SPREAD_RATIO.0, SPREAD_RATIO.1);
            let weighted_average_neighbour_score = (cardinal_sum * 3 + ordinal_sum * 2) / (5 * 8);
            let to_add = (weighted_average_neighbour_score * SPREAD_RATIO.0) / SPREAD_RATIO.1;
            let dest_cell = self.grid.get_checked_mut(dest_coord);
            let score_before_adding = (dest_cell.score as u128 * REMAIN_RATIO.0) / REMAIN_RATIO.1;
            dest_cell.next_score =
                (((score_before_adding + to_add) * FADE_RATIO.0) / FADE_RATIO.1).min(u64::max_value() as u128) as u64;
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
    fn tick(&mut self, game_data: &GameData) {
        self.spread(game_data);
    }
    fn is_empty(&self) -> bool {
        self.count == 0
    }
}

impl ParticleSystem {
    pub fn new(size: Size) -> Self {
        Self {
            active_particles: Vec::new(),
            next_active_particles: Vec::new(),
            trails: Trails {
                grid: Grid::new_fn(size, |_| TrailCell::new()),
                count: 0,
            },
        }
    }
    pub fn register(&mut self, particle: Particle) {
        self.active_particles.push(particle);
    }
    pub fn tick(&mut self, game_data: &mut GameData) {
        for mut particle in self.active_particles.drain(..) {
            match particle.tick(&mut self.trails, game_data) {
                ControlFlow::Break => (),
                ControlFlow::Continue => self.next_active_particles.push(particle),
            }
        }
        mem::swap(&mut self.active_particles, &mut self.next_active_particles);
        self.trails.tick(game_data);
    }
    pub fn is_empty(&self) -> bool {
        self.active_particles.is_empty() && self.trails.is_empty()
    }
    pub fn particles(&self) -> &[Particle] {
        &self.active_particles
    }
    pub fn trails_grid(&self) -> &Grid<TrailCell> {
        &self.trails.grid
    }
}
