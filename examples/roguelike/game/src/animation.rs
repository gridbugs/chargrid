#![allow(dead_code)]
use crate::circle;
use crate::data::{GameData, Id, ProjectileMoveError};
use line_2d::{Config as LineConfig, Coord, LineSegment, NodeIter as LineSegmentIter};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::mem;
use std::time::Duration;

pub type Animation = Box<dyn Animate>;

pub enum AnimateResult {
    Continue,
    Break,
}

#[typetag::serde(tag = "type")]
pub trait Animate {
    fn step(&mut self, since_last_frame: Duration, data: &mut GameData) -> AnimateResult;
    fn cleanup(&mut self, data: &mut GameData);
}

#[typetag::serde(tag = "type")]
pub trait AnimationFactory {
    fn make(&mut self, data: &mut GameData) -> Animation;
}

#[typetag::serde(tag = "type")]
pub trait AnimationFactoryArgCoord {
    fn make(&mut self, coord: Coord, data: &mut GameData) -> Animation;
}

#[derive(Serialize, Deserialize)]
pub struct SingleProjectile {
    path_iter: LineSegmentIter,
    step_duration: Duration,
    until_next_step: Duration,
    entity_id: Id,
}

#[derive(Serialize, Deserialize)]
pub struct ExplodeFactory {
    offsets: Vec<Coord>,
}

impl ExplodeFactory {
    pub fn new<R: Rng>(radius: i32, rng: &mut R) -> Self {
        let mut offsets = Vec::new();
        for _ in 0..256 {
            let length = rng.gen_range((radius - 2).max(0), (radius + 2).max(1));
            let coord = circle::random_coord_with_cardinal_length(length, rng);
            offsets.push(coord);
        }
        Self { offsets }
    }
}

#[typetag::serde]
impl AnimationFactoryArgCoord for ExplodeFactory {
    fn make(&mut self, coord: Coord, data: &mut GameData) -> Animation {
        let mut particles = Vec::new();
        for offset in self.offsets.iter() {
            let line_segment = LineSegment::new(coord, coord + offset);
            let step_duration = Duration::from_millis(20);
            let single_projectile: Animation = Box::new(SingleProjectile::new(line_segment, step_duration, data));
            particles.push(single_projectile);
        }
        Box::new(Parallel::new(particles))
    }
}

#[derive(Serialize, Deserialize)]
pub struct SingleProjectileFactory {
    path: LineSegment,
    step_duration: Duration,
}

impl SingleProjectileFactory {
    pub fn new(path: LineSegment, step_duration: Duration) -> Self {
        Self { path, step_duration }
    }
}

#[typetag::serde]
impl AnimationFactory for SingleProjectileFactory {
    fn make(&mut self, data: &mut GameData) -> Animation {
        Box::new(SingleProjectile::new(self.path, self.step_duration, data))
    }
}

impl SingleProjectile {
    pub fn new(path: LineSegment, step_duration: Duration, data: &mut GameData) -> Self {
        let path_iter = path.config_node_iter(LineConfig {
            exclude_start: true,
            exclude_end: false,
        });
        let start_coord = path_iter.clone().next().map(|node| node.coord).unwrap_or(path.start());
        let entity_id = data.make_projectile(start_coord);
        Self {
            path_iter,
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
        } else if let Some(node) = self.path_iter.next() {
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
                Err(ProjectileMoveError::DestinationSolid) => AnimateResult::Break,
            }
        } else {
            AnimateResult::Break
        }
    }
    fn cleanup(&mut self, data: &mut GameData) {
        data.remove_projectile(self.entity_id);
    }
}

#[derive(Serialize, Deserialize)]
pub enum SingleProjectileThen {
    First {
        single_projectile: SingleProjectile,
        second_factory: Box<dyn AnimationFactoryArgCoord>,
    },
    Second(Animation),
}

impl SingleProjectileThen {
    pub fn new(single_projectile: SingleProjectile, second_factory: Box<dyn AnimationFactoryArgCoord>) -> Self {
        Self::First {
            single_projectile,
            second_factory,
        }
    }
}

#[typetag::serde]
impl Animate for SingleProjectileThen {
    fn step(&mut self, since_last_frame: Duration, data: &mut GameData) -> AnimateResult {
        match self {
            Self::First {
                single_projectile,
                second_factory,
            } => {
                let current_coord = single_projectile.path_iter.current();
                match single_projectile.step(since_last_frame, data) {
                    AnimateResult::Continue => (),
                    AnimateResult::Break => {
                        single_projectile.cleanup(data);
                        *self = Self::Second(second_factory.make(current_coord, data));
                    }
                }
                AnimateResult::Continue
            }
            Self::Second(animation) => animation.step(since_last_frame, data),
        }
    }
    fn cleanup(&mut self, data: &mut GameData) {
        match self {
            Self::First {
                single_projectile,
                second_factory: _,
            } => single_projectile.cleanup(data),
            Self::Second(animation) => animation.cleanup(data),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum LazyThen {
    First {
        animation: Animation,
        second_factory: Box<dyn AnimationFactory>,
    },
    Second(Animation),
}

impl LazyThen {
    pub fn new(animation: Animation, second_factory: Box<dyn AnimationFactory>) -> Self {
        Self::First {
            animation,
            second_factory,
        }
    }
}

#[typetag::serde]
impl Animate for LazyThen {
    fn step(&mut self, since_last_frame: Duration, data: &mut GameData) -> AnimateResult {
        match self {
            Self::First {
                animation,
                second_factory,
            } => {
                match animation.step(since_last_frame, data) {
                    AnimateResult::Continue => (),
                    AnimateResult::Break => {
                        animation.cleanup(data);
                        *self = Self::Second(second_factory.make(data));
                    }
                }
                AnimateResult::Continue
            }
            Self::Second(animation) => animation.step(since_last_frame, data),
        }
    }
    fn cleanup(&mut self, data: &mut GameData) {
        match self {
            Self::First {
                animation,
                second_factory: _,
            } => {
                animation.cleanup(data);
            }
            Self::Second(animation) => animation.cleanup(data),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Schedule {
    animations: Vec<Animation>,
    next_animations: Vec<Animation>,
}

impl Schedule {
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
            next_animations: Vec::new(),
        }
    }
    pub fn register(&mut self, animation: Animation) {
        self.animations.push(animation);
    }
    pub fn is_empty(&self) -> bool {
        self.animations.is_empty()
    }
    pub fn tick(&mut self, since_last_tick: Duration, data: &mut GameData) {
        for mut animation in self.animations.drain(..) {
            match animation.step(since_last_tick, data) {
                AnimateResult::Break => animation.cleanup(data),
                AnimateResult::Continue => self.next_animations.push(animation),
            }
        }
        mem::swap(&mut self.animations, &mut self.next_animations);
    }
}

#[derive(Serialize, Deserialize)]
pub struct Parallel {
    schedule: Schedule,
}

impl Parallel {
    pub fn new(animations: Vec<Animation>) -> Self {
        let schedule = Schedule {
            animations,
            next_animations: Vec::new(),
        };
        Self { schedule }
    }
}

#[typetag::serde]
impl Animate for Parallel {
    fn step(&mut self, since_last_frame: Duration, data: &mut GameData) -> AnimateResult {
        self.schedule.tick(since_last_frame, data);
        if self.schedule.is_empty() {
            AnimateResult::Break
        } else {
            AnimateResult::Continue
        }
    }
    fn cleanup(&mut self, data: &mut GameData) {
        for mut animation in self.schedule.animations.drain(..) {
            animation.cleanup(data);
        }
    }
}
