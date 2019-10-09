use line_2d::{Coord, LineSegment, NodeIter};
use serde::{Deserialize, Serialize};
use std::time::Duration;

// The frame duration is fixed here for consistent behaviour across
// variations in frame-rate.
const FRAME_MICROS: i64 = 1_000_000 / 60;

#[derive(Serialize, Deserialize, Debug)]
pub struct Projectile<V> {
    cardinal_step_duration_micros: i64,
    ordinal_step_duration_micros: i64,
    path_iter: NodeIter,
    path: LineSegment,
    budget_micros: i64,
    pub value: V,
}

pub enum Step {
    MoveTo(Coord),
    AtDestination,
}

pub struct ProjectileFrameIter<'a, V> {
    projectile: &'a mut Projectile<V>,
}

impl<'a, V> Iterator for ProjectileFrameIter<'a, V> {
    type Item = Step;
    fn next(&mut self) -> Option<Self::Item> {
        self.projectile.step()
    }
}

impl<V> Projectile<V> {
    fn ordinal_duration_from_cardinal_duration(duration_micros: i64) -> i64 {
        const SQRT_2_X_1_000_000: i64 = 1414214;
        let diagonal_micros = (duration_micros * SQRT_2_X_1_000_000) / 1_000_000;
        diagonal_micros
    }
    pub fn new(path: LineSegment, step_duration: Duration, value: V) -> Self {
        let cardinal_step_duration_micros = step_duration.as_micros() as i64;
        Self {
            cardinal_step_duration_micros,
            ordinal_step_duration_micros: Self::ordinal_duration_from_cardinal_duration(cardinal_step_duration_micros),
            path_iter: path.node_iter(),
            path,
            budget_micros: FRAME_MICROS,
            value,
        }
    }
    pub fn coord(&self) -> Coord {
        self.path_iter.current()
    }
    pub fn frame_iter(&mut self) -> ProjectileFrameIter<V> {
        self.replenish();
        ProjectileFrameIter { projectile: self }
    }
    fn replenish(&mut self) {
        self.budget_micros += FRAME_MICROS;
    }
    fn step(&mut self) -> Option<Step> {
        if self.budget_micros >= 0 {
            if let Some(node) = self.path_iter.next() {
                let step_duration = if node.prev.is_cardinal() {
                    self.cardinal_step_duration_micros
                } else {
                    self.ordinal_step_duration_micros
                };
                self.budget_micros -= step_duration;
                Some(Step::MoveTo(node.coord))
            } else {
                Some(Step::AtDestination)
            }
        } else {
            None
        }
    }
}
