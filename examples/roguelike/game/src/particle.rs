use line_2d::{Coord, Direction, InfiniteStepIter};
use serde::{Deserialize, Serialize};
use std::time::Duration;

// The frame duration is fixed here for consistent behaviour across
// variations in frame-rate.
const FRAME_MICROS: i64 = 1_000_000 / 60;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Particle {
    cardinal_step_duration_micros: i64,
    ordinal_step_duration_micros: i64,
    step_iter: InfiniteStepIter,
    budget_micros: i64,
}

pub struct ParticleFrameIter<'a> {
    particle: &'a mut Particle,
}

impl<'a> Iterator for ParticleFrameIter<'a> {
    type Item = Direction;
    fn next(&mut self) -> Option<Self::Item> {
        self.particle.step()
    }
}

impl Particle {
    fn ordinal_duration_from_cardinal_duration(duration_micros: i64) -> i64 {
        const SQRT_2_X_1_000_000: i64 = 1414214;
        let diagonal_micros = (duration_micros * SQRT_2_X_1_000_000) / 1_000_000;
        diagonal_micros
    }
    pub fn new(delta: Coord, step_duration: Duration) -> Self {
        let cardinal_step_duration_micros = step_duration.as_micros() as i64;
        Self {
            cardinal_step_duration_micros,
            ordinal_step_duration_micros: Self::ordinal_duration_from_cardinal_duration(cardinal_step_duration_micros),
            step_iter: InfiniteStepIter::new(delta),
            budget_micros: 0,
        }
    }
    pub fn frame_iter(&mut self) -> ParticleFrameIter {
        self.replenish();
        ParticleFrameIter { particle: self }
    }
    fn replenish(&mut self) {
        self.budget_micros += FRAME_MICROS;
    }
    fn step(&mut self) -> Option<Direction> {
        if self.budget_micros >= 0 {
            if let Some(direction) = self.step_iter.next() {
                let step_duration = if direction.is_cardinal() {
                    self.cardinal_step_duration_micros
                } else {
                    self.ordinal_step_duration_micros
                };
                self.budget_micros -= step_duration;
                Some(direction)
            } else {
                None
            }
        } else {
            None
        }
    }
}
