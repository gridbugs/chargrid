use crate::data::{GameData, Id, ProjectileMoveError};
use line_2d::{Coord, NodeIter as LineSegmentIter};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub enum AnimateResult<R> {
    Continue,
    Return(R),
}

impl<R> AnimateResult<R> {
    fn return_unit(&self) -> AnimateResult<()> {
        match self {
            Self::Continue => AnimateResult::Continue,
            Self::Return(_) => AnimateResult::Return(()),
        }
    }
}

#[typetag::serde(tag = "type")]
pub trait Animate {
    fn tick(&mut self, since_last: Duration, data: &mut GameData) -> AnimateResult<()>;
    fn cleanup(&mut self, data: &mut GameData);
}

#[typetag::serde(tag = "type")]
pub trait AnimateReturnCoord {
    fn tick_return_coord(&mut self, since_last: Duration, data: &mut GameData) -> AnimateResult<Coord>;
}

#[derive(Serialize, Deserialize)]
pub struct Projectile {
    path_iter: LineSegmentIter,
    step_duration: Duration,
    until_next_step: Duration,
    entity_id: Id,
}

#[typetag::serde]
impl Animate for Projectile {
    fn tick(&mut self, since_last: Duration, data: &mut GameData) -> AnimateResult<()> {
        self.tick_return_coord(since_last, data).return_unit()
    }
    fn cleanup(&mut self, data: &mut GameData) {
        data.remove_projectile(self.entity_id);
    }
}

#[typetag::serde]
impl AnimateReturnCoord for Projectile {
    fn tick_return_coord(&mut self, since_last: Duration, data: &mut GameData) -> AnimateResult<Coord> {
        if let Some(remaining_until_next_step) = self.until_next_step.checked_sub(since_last) {
            self.until_next_step = remaining_until_next_step;
            AnimateResult::Continue
        } else {
            let current_coord = self.path_iter.current();
            if let Some(node) = self.path_iter.next() {
                match data.move_projectile(self.entity_id, node.coord) {
                    Ok(()) => {
                        self.until_next_step = if node.prev.is_cardinal() {
                            self.step_duration
                        } else {
                            let micros = self.step_duration.as_micros();
                            let diagonal_micros = (micros * 1414) / 1000;
                            Duration::from_micros(diagonal_micros as u64)
                        };
                        AnimateResult::Continue
                    }
                    Err(ProjectileMoveError::DestinationSolid) => AnimateResult::Return(current_coord),
                }
            } else {
                AnimateResult::Return(current_coord)
            }
        }
    }
}
