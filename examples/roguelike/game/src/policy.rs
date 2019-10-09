use crate::entity_grid::{Entity, IdTrait, ManyPerCell, OnePerCell};
use direction::CardinalDirection;
use grid_2d::{Coord, Size};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct WallId(u64);
#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct CharacterId(u64);
#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ProjectileId(u64);

impl IdTrait for WallId {
    fn from_u64(u64: u64) -> Self {
        Self(u64)
    }
    fn to_u64(self) -> u64 {
        self.0
    }
}

impl IdTrait for CharacterId {
    fn from_u64(u64: u64) -> Self {
        Self(u64)
    }
    fn to_u64(self) -> u64 {
        self.0
    }
}

impl IdTrait for ProjectileId {
    fn from_u64(u64: u64) -> Self {
        Self(u64)
    }
    fn to_u64(self) -> u64 {
        self.0
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum CharacterTile {
    Player,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Character {
    tile: CharacterTile,
}

impl Character {
    pub fn tile(&self) -> CharacterTile {
        self.tile
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Wall {}

#[derive(Serialize, Deserialize, Debug)]
pub struct Projectile {}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameData {
    characters: OnePerCell<CharacterId, Character>,
    walls: OnePerCell<WallId, Wall>,
    projectiles: ManyPerCell<ProjectileId, Projectile>,
}

impl GameData {
    pub fn new(size: Size) -> Self {
        let characters = OnePerCell::new(size);
        let walls = OnePerCell::new(size);
        let projectiles = ManyPerCell::new(size);
        Self {
            characters,
            walls,
            projectiles,
        }
    }
    pub fn spawn_wall(&mut self, coord: Coord) {
        self.walls.spawn_entity(Wall {}, coord);
    }
    pub fn spawn_player(&mut self, coord: Coord) -> CharacterId {
        self.characters
            .spawn_entity(
                Character {
                    tile: CharacterTile::Player,
                },
                coord,
            )
            .unwrap()
    }
    pub fn spawn_projectile(&mut self, coord: Coord) -> ProjectileId {
        self.projectiles.spawn_entity(Projectile {}, coord).unwrap()
    }
    pub fn character_walk_in_direction(&mut self, id: CharacterId, direction: CardinalDirection) {}
}
