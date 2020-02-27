use crate::visibility::Light;
pub use crate::world::{
    explosion_spec,
    spatial::{Layer, Location},
};
use ecs::ecs_components;
use rgb24::Rgb24;
use serde::{Deserialize, Serialize};

ecs_components! {
    components {
        tile: Tile,
        opacity: u8,
        solid: (),
        realtime: (),
        blocks_gameplay: (),
        light: Light,
        on_collision: OnCollision,
        colour_hint: Rgb24,
        npc: Npc,
        character: (),
        collides_with: CollidesWith,
        projectile_damage: ProjectileDamage,
        hit_points: HitPoints,
        blood: (),
        player: (),
        ignore_lighting: (),
        door_state: DoorState,
        stairs: (),
    }
}
pub use components::Components;
pub use components::EntityData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Player,
    Wall,
    Floor,
    Carpet,
    Window,
    Bullet,
    Smoke,
    ExplosionFlame,
    FormerHuman,
    Human,
    Star,
    Space,
    DoorClosed,
    DoorOpen,
    Stairs,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Disposition {
    Hostile,
    Afraid,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Npc {
    pub disposition: Disposition,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OnCollision {
    Explode(explosion_spec::Explosion),
    Remove,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CollidesWith {
    pub solid: bool,
    pub character: bool,
}

impl Default for CollidesWith {
    fn default() -> Self {
        Self {
            solid: true,
            character: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ProjectileDamage {
    pub hit_points: u32,
    pub push_back: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HitPoints {
    pub current: u32,
    pub max: u32,
}

impl HitPoints {
    pub fn new_full(max: u32) -> Self {
        Self { current: max, max }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DoorState {
    Open,
    Closed,
}
