use crate::data::{GameData, Id, ProjectileMoveError};
use grid_2d::Grid;
use line_2d::{Coord, LineSegment, NodeIter};
use rgb24::Rgb24;
use std::time::Duration;

const FRAME_DURATION: Duration = Duration::from_micros(1_000_000 / 60);

struct Particle {
    cardinal_step_duration: Duration,
    ordinal_step_duration: Duration,
    until_next_step: Duration,
    path_iter: NodeIter,
    path: LineSegment,
}

struct TrailCell {
    from: Rgb24,
    to: Rgb24,
    total_duration: Duration,
    since_start: Duration,
}

pub struct Trails {
    grid: Grid<Option<TrailCell>>,
    count: u64,
}

pub struct ParticleSystem {
    active_particles: Vec<Particle>,
    trails: Trails,
}

enum ControlFlow {
    Continue,
    Break,
}

impl Particle {
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
