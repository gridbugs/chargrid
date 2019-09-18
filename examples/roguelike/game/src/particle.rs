use crate::data::GameData;
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
}

#[derive(Serialize, Deserialize)]
struct TrailCell {
    from: Rgb24,
    to: Rgb24,
    total_duration: Duration,
    since_start: Duration,
}

#[derive(Serialize, Deserialize)]
pub struct Trails {
    grid: Grid<Option<TrailCell>>,
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
    pub fn new(path: LineSegment, step_duration: Duration) -> Self {
        Self {
            cardinal_step_duration: step_duration,
            ordinal_step_duration: Self::ordinal_duration_from_cardinal_duration(step_duration),
            until_next_step: Duration::from_millis(0),
            path_iter: path.node_iter(),
            path,
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
                    break ControlFlow::Break;
                }
            }
        }
    }
    pub fn coord(&self) -> Coord {
        self.path_iter.current()
    }
}

impl Trails {
    fn register(&mut self, coord: Coord) {
        if let Some(cell) = self.grid.get_mut(coord) {
            if cell.is_none() {
                self.count += 1;
                *cell = Some(TrailCell {
                    from: Rgb24::new(255, 255, 255),
                    to: Rgb24::new(0, 0, 0),
                    total_duration: Duration::from_millis(500),
                    since_start: Duration::from_millis(0),
                });
            }
        }
    }
}

impl ParticleSystem {
    pub fn new(size: Size) -> Self {
        Self {
            active_particles: Vec::new(),
            next_active_particles: Vec::new(),
            trails: Trails {
                grid: Grid::new_fn(size, |_| None),
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
    }
    pub fn is_empty(&self) -> bool {
        self.active_particles.is_empty()
    }
    pub fn particles(&self) -> &[Particle] {
        &self.active_particles
    }
}
