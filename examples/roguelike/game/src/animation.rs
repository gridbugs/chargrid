use crate::data::{GameData, Id, ProjectileMoveError};
use line_2d::{Config as LineConfig, Iter as LineSegmentIter, LineSegment};
use serde::{Deserialize, Serialize};
use std::mem;
use std::time::Duration;

pub enum AnimateResult {
    Continue,
    Break,
}

#[typetag::serde(tag = "type")]
pub trait Animate {
    fn step(&mut self, since_last_frame: Duration, data: &mut GameData) -> AnimateResult;
}

#[derive(Serialize, Deserialize)]
pub struct SingleProjectile {
    path_iter: LineSegmentIter,
    step_duration: Duration,
    until_next_step: Duration,
    entity_id: Id,
}

impl SingleProjectile {
    pub fn new(path: LineSegment, step_duration: Duration, data: &mut GameData) -> Self {
        let entity_id = data.make_projectile(path.start());
        Self {
            path_iter: path.config_iter(LineConfig {
                exclude_start: true,
                exclude_end: false,
            }),
            step_duration,
            until_next_step: Duration::from_millis(0),
            entity_id,
        }
    }
}

#[typetag::serde]
impl Animate for SingleProjectile {
    fn step(&mut self, since_last_frame: Duration, data: &mut GameData) -> AnimateResult {
        if let Some(remaining_until_next_step) = self.until_next_step.checked_sub(since_last_frame) {
            self.until_next_step = remaining_until_next_step;
            AnimateResult::Continue
        } else if let Some(coord) = self.path_iter.next() {
            match data.move_projectile(self.entity_id, coord) {
                Ok(()) => {
                    self.until_next_step = self.step_duration;
                    AnimateResult::Continue
                }
                Err(ProjectileMoveError::DestinationSolid) => {
                    data.remove_projectile(self.entity_id);
                    AnimateResult::Break
                }
            }
        } else {
            data.remove_projectile(self.entity_id);
            AnimateResult::Break
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
enum ThenStage {
    First,
    Second,
}

#[derive(Serialize, Deserialize)]
pub struct Then {
    first: Box<dyn Animate>,
    second: Box<dyn Animate>,
    stage: ThenStage,
}

impl Then {
    pub fn new(first: Box<dyn Animate>, second: Box<dyn Animate>) -> Self {
        Self {
            first,
            second,
            stage: ThenStage::First,
        }
    }
}

#[typetag::serde]
impl Animate for Then {
    fn step(&mut self, since_last_frame: Duration, data: &mut GameData) -> AnimateResult {
        match self.stage {
            ThenStage::First => {
                match self.first.step(since_last_frame, data) {
                    AnimateResult::Continue => (),
                    AnimateResult::Break => self.stage = ThenStage::Second,
                }
                AnimateResult::Continue
            }
            ThenStage::Second => self.second.step(since_last_frame, data),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Schedule {
    animations: Vec<Box<dyn Animate>>,
    next_animations: Vec<Box<dyn Animate>>,
}

impl Schedule {
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
            next_animations: Vec::new(),
        }
    }
    pub fn register(&mut self, animation: Box<dyn Animate>) {
        self.animations.push(animation);
    }
    pub fn is_empty(&self) -> bool {
        self.animations.is_empty()
    }
    pub fn tick(&mut self, since_last_tick: Duration, data: &mut GameData) {
        for mut animation in self.animations.drain(..) {
            match animation.step(since_last_tick, data) {
                AnimateResult::Break => (),
                AnimateResult::Continue => self.next_animations.push(animation),
            }
        }
        mem::swap(&mut self.animations, &mut self.next_animations);
    }
}
