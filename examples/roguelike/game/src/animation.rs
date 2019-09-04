use crate::data::{GameData, Id, ProjectileMoveError};
use line_2d::{Config as LineConfig, Iter as LineSegmentIter, LineSegment};
use serde::{Deserialize, Serialize};
use std::mem;
use std::time::Duration;

pub enum AnimateResult {
    Complete,
    NextStepIn(Duration),
}

#[typetag::serde(tag = "type")]
pub trait Animate {
    fn step(&mut self, data: &mut GameData) -> AnimateResult;
}

#[derive(Serialize, Deserialize)]
pub struct SingleProjectile {
    path_iter: LineSegmentIter,
    step_duration: Duration,
    entity_id: Id,
}

impl SingleProjectile {
    pub fn new(path: LineSegment, step_duration: Duration, data: &mut GameData) -> Self {
        let entity_id = data.make_projectile(path.start);
        Self {
            path_iter: path.config_iter(LineConfig {
                exclude_start: true,
                exclude_end: false,
            }),
            step_duration,
            entity_id,
        }
    }
}

#[typetag::serde]
impl Animate for SingleProjectile {
    fn step(&mut self, data: &mut GameData) -> AnimateResult {
        if let Some(coord) = self.path_iter.next() {
            match data.move_projectile(self.entity_id, coord) {
                Ok(()) => return AnimateResult::NextStepIn(self.step_duration),
                Err(ProjectileMoveError::DestinationSolid) => (),
            }
        }
        data.remove_projectile(self.entity_id);
        AnimateResult::Complete
    }
}

#[derive(Serialize, Deserialize)]
struct ScheduleEntry {
    animation: Box<dyn Animate>,
    until_next_step: Duration,
}

struct ScheduleAgain;

impl ScheduleEntry {
    fn tick(&mut self, data: &mut GameData, mut since_last_tick: Duration) -> Option<ScheduleAgain> {
        while let Some(remaining_since_last_tick) = since_last_tick.checked_sub(self.until_next_step) {
            match self.animation.step(data) {
                AnimateResult::NextStepIn(until_next_step) => self.until_next_step = until_next_step,
                AnimateResult::Complete => return None,
            }
            since_last_tick = remaining_since_last_tick;
        }
        self.until_next_step -= since_last_tick;
        Some(ScheduleAgain)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Schedule {
    animations: Vec<ScheduleEntry>,
    next_animations: Vec<ScheduleEntry>,
}

impl Schedule {
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
            next_animations: Vec::new(),
        }
    }
    pub fn register(&mut self, animation: Box<dyn Animate>) {
        let entry = ScheduleEntry {
            animation,
            until_next_step: Duration::from_millis(0),
        };
        self.animations.push(entry);
    }
    pub fn tick(&mut self, data: &mut GameData, since_last_tick: Duration) {
        for mut entry in self.animations.drain(..) {
            match entry.tick(data, since_last_tick) {
                None => (),
                Some(ScheduleAgain) => self.next_animations.push(entry),
            }
        }
        mem::swap(&mut self.animations, &mut self.next_animations);
    }
    pub fn is_empty(&self) -> bool {
        self.animations.is_empty()
    }
}
