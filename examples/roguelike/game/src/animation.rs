use crate::circle;
use crate::data::{GameData, Id, ProjectileMoveError};
use line_2d::{Config as LineConfig, Coord, LineSegment, NodeIter as LineSegmentIter};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::mem;
use std::time::Duration;

pub enum Control<R> {
    Continue,
    Return(R),
}

pub struct AnimateResult<R> {
    pub duration: Duration,
    pub control: Control<R>,
}

impl<R> AnimateResult<R> {
    fn return_unit(self) -> AnimateResult<()> {
        let AnimateResult { duration, control } = self;
        AnimateResult {
            duration,
            control: match control {
                Control::Continue => Control::Continue,
                Control::Return(_) => Control::Return(()),
            },
        }
    }
}

#[typetag::serde(tag = "type")]
pub trait Animate {
    fn tick(&mut self, since_last_tick: Duration, data: &mut GameData) -> AnimateResult<()>;
    fn cleanup(&mut self, data: &mut GameData);
}

#[typetag::serde(tag = "type")]
pub trait AnimateReturnCoord {
    fn tick_return_coord(&mut self, since_last_tick: Duration, data: &mut GameData) -> AnimateResult<Coord>;
    fn cleanup_return_coord(&mut self, data: &mut GameData);
}

pub type Animation = Box<dyn Animate>;
pub type AnimationReturnCoord = Box<dyn AnimateReturnCoord>;

#[typetag::serde(tag = "type")]
pub trait AnimationFactoryArgCoord {
    fn make_arg_coord(&mut self, data: &mut GameData, coord: Coord) -> Animation;
}

#[derive(Serialize, Deserialize)]
pub struct ExplodeFactory {
    offsets: Vec<Coord>,
}

impl ExplodeFactory {
    pub fn new<R: Rng>(radius: i32, rng: &mut R) -> Self {
        let mut offsets = Vec::new();
        for i in 0..=255 {
            let length = rng.gen_range((radius - 2).max(0), (radius + 2).max(1));
            let coord = circle::scale_to_cardinal_length(circle::coord(i), length);
            offsets.push(coord);
        }
        Self { offsets }
    }
}

#[typetag::serde]
impl AnimationFactoryArgCoord for ExplodeFactory {
    fn make_arg_coord(&mut self, data: &mut GameData, coord: Coord) -> Animation {
        let mut particles = Vec::new();
        for offset in self.offsets.iter() {
            let line_segment = LineSegment::new(coord, coord + offset);
            let step_duration = Duration::from_millis(50);
            let single_projectile: Animation = Box::new(Saturate::new(Box::new(Projectile::new(
                line_segment,
                step_duration,
                data,
            ))));
            particles.push(single_projectile);
        }
        Box::new(Parallel::new(particles))
    }
}

#[derive(Serialize, Deserialize)]
pub struct Saturate {
    until_next_tick: Duration,
    animation: Animation,
}

impl Saturate {
    pub fn new(animation: Animation) -> Self {
        Self {
            animation,
            until_next_tick: Duration::from_millis(0),
        }
    }
}

#[typetag::serde]
impl Animate for Saturate {
    fn tick(&mut self, since_last_tick: Duration, data: &mut GameData) -> AnimateResult<()> {
        if let Some(next_until_next_tick) = self.until_next_tick.checked_sub(since_last_tick) {
            self.until_next_tick = next_until_next_tick;
            AnimateResult {
                duration: since_last_tick,
                control: Control::Continue,
            }
        } else {
            let mut total_duration = Duration::from_millis(0);
            let mut remaining = since_last_tick;
            loop {
                let AnimateResult { duration, control } = self.animation.tick(since_last_tick, data);
                total_duration += duration;
                match control {
                    Control::Return(()) => {
                        break AnimateResult {
                            control: Control::Return(()),
                            duration: total_duration,
                        }
                    }
                    Control::Continue => {
                        if let Some(next_remaining) = remaining.checked_sub(duration) {
                            remaining = next_remaining;
                        } else {
                            self.until_next_tick = duration - remaining;
                            break AnimateResult {
                                control: Control::Continue,
                                duration: duration - remaining,
                            };
                        }
                    }
                }
            }
        }
    }
    fn cleanup(&mut self, data: &mut GameData) {
        self.animation.cleanup(data);
    }
}

#[derive(Serialize, Deserialize)]
pub struct Projectile {
    path_iter: LineSegmentIter,
    cardinal_step_duration: Duration,
    ordinal_step_duration: Duration,
    entity_id: Id,
}

impl Projectile {
    pub fn new(path: LineSegment, step_duration: Duration, data: &mut GameData) -> Self {
        let path_iter = path.config_node_iter(LineConfig {
            exclude_start: false,
            exclude_end: false,
        });
        let entity_id = data.make_projectile(path.start());
        let micros = step_duration.as_micros();
        let diagonal_micros = (micros * 1414) / 1000;
        let ordinal_step_duration = Duration::from_micros(diagonal_micros as u64);
        Self {
            path_iter,
            cardinal_step_duration: step_duration,
            ordinal_step_duration,
            entity_id,
        }
    }
}

#[typetag::serde]
impl Animate for Projectile {
    fn tick(&mut self, since_last_tick: Duration, data: &mut GameData) -> AnimateResult<()> {
        self.tick_return_coord(since_last_tick, data).return_unit()
    }
    fn cleanup(&mut self, data: &mut GameData) {
        self.cleanup_return_coord(data);
    }
}

#[typetag::serde]
impl AnimateReturnCoord for Projectile {
    fn tick_return_coord(&mut self, _since_last_tick: Duration, data: &mut GameData) -> AnimateResult<Coord> {
        let current_coord = self.path_iter.current();
        if let Some(node) = self.path_iter.next() {
            match data.move_projectile(self.entity_id, node.coord) {
                Ok(()) => {
                    let duration = if node.prev.is_cardinal() {
                        self.cardinal_step_duration
                    } else {
                        self.ordinal_step_duration
                    };
                    AnimateResult {
                        duration,
                        control: Control::Continue,
                    }
                }
                Err(ProjectileMoveError::DestinationSolid) => AnimateResult {
                    duration: Duration::from_millis(0),
                    control: Control::Return(current_coord),
                },
            }
        } else {
            AnimateResult {
                duration: Duration::from_millis(0),
                control: Control::Return(current_coord),
            }
        }
    }
    fn cleanup_return_coord(&mut self, data: &mut GameData) {
        data.remove_projectile(self.entity_id);
    }
}

#[derive(Serialize, Deserialize)]
pub struct Schedule {
    parallel: Parallel,
}

impl Schedule {
    pub fn new() -> Self {
        Self {
            parallel: Parallel::new(vec![]),
        }
    }
    pub fn register(&mut self, animation: Animation) {
        self.parallel.animations.push(animation);
    }
    pub fn is_empty(&self) -> bool {
        self.parallel.is_empty()
    }
    pub fn tick(&mut self, since_last_tick: Duration, data: &mut GameData) {
        self.parallel.tick(since_last_tick, data);
    }
}

#[derive(Serialize, Deserialize)]
pub struct Parallel {
    animations: Vec<Animation>,
    next_animations: Vec<Animation>,
}

impl Parallel {
    pub fn new(animations: Vec<Animation>) -> Self {
        Self {
            animations,
            next_animations: Vec::new(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.animations.is_empty()
    }
}

#[typetag::serde]
impl Animate for Parallel {
    fn tick(&mut self, since_last_tick: Duration, data: &mut GameData) -> AnimateResult<()> {
        if self.is_empty() {
            AnimateResult {
                control: Control::Return(()),
                duration: Duration::from_millis(0),
            }
        } else {
            let mut max_duration = Duration::from_millis(0);
            for mut animation in self.animations.drain(..) {
                let AnimateResult { control, duration } = animation.tick(since_last_tick, data);
                max_duration = max_duration.max(duration);
                match control {
                    Control::Return(()) => animation.cleanup(data),
                    Control::Continue => self.next_animations.push(animation),
                }
            }
            mem::swap(&mut self.animations, &mut self.next_animations);
            AnimateResult {
                control: Control::Continue,
                duration: max_duration,
            }
        }
    }
    fn cleanup(&mut self, data: &mut GameData) {
        for mut animation in self.animations.drain(..) {
            animation.cleanup(data);
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum AndThenCoord {
    First {
        current: AnimationReturnCoord,
        next: Box<dyn AnimationFactoryArgCoord>,
    },
    Second(Animation),
}

#[typetag::serde]
impl Animate for AndThenCoord {
    fn tick(&mut self, since_last_tick: Duration, data: &mut GameData) -> AnimateResult<()> {
        match self {
            Self::First { current, next } => {
                let AnimateResult { duration, control } = current.tick_return_coord(since_last_tick, data);
                match control {
                    Control::Continue => (),
                    Control::Return(coord) => {
                        current.cleanup_return_coord(data);
                        let animation = next.make_arg_coord(data, coord);
                        *self = Self::Second(animation);
                    }
                }
                AnimateResult {
                    duration,
                    control: Control::Continue,
                }
            }
            Self::Second(animation) => animation.tick(since_last_tick, data),
        }
    }

    fn cleanup(&mut self, data: &mut GameData) {
        match self {
            Self::First { current, .. } => current.cleanup_return_coord(data),
            Self::Second(animation) => animation.cleanup(data),
        }
    }
}

impl AndThenCoord {
    pub fn new(current: AnimationReturnCoord, next: Box<dyn AnimationFactoryArgCoord>) -> Self {
        Self::First { current, next }
    }
}
