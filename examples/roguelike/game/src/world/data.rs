use crate::visibility::Light;
pub use crate::world::spatial_grid::{Layer, Location};
use ecs::ecs_components;
use rgb24::Rgb24;
use serde::{Deserialize, Serialize};

ecs_components! {
    components {
        location: Location,
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
        colides_with: ColidesWith,
    }
}
pub use components::Components;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Player,
    Wall,
    Floor,
    Carpet,
    Bullet,
    Smoke,
    ExplosionFlame,
    FormerHuman,
    Human,
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
    Explode,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ColidesWith {
    pub solid: bool,
    pub character: bool,
}

impl Default for ColidesWith {
    fn default() -> Self {
        Self {
            solid: true,
            character: false,
        }
    }
}
