use crate::data::{GameData, Id, ProjectileMoveError};
use line_2d::{Config as LineConfig, Coord, LineSegment, NodeIter as LineSegmentIter};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize)]
pub struct Projectile {
    path_iter: LineSegmentIter,
    cardinal_step_duration: Duration,
    ordinal_step_duration: Duration,
    entity_id: Id,
}

#[derive(Serialize, Deserialize)]
pub enum Animation {
    Projectile(Projectile),
}
