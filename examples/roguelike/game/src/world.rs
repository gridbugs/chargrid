use crate::entity_grid::{EntityByCoord, EntityGrid, IdTrait, RawId};
use crate::realtime_gas::{GasGrid, GasRatio, GasSpec};
use crate::realtime_projectile::{Projectile, Step};
use direction::CardinalDirection;
use grid_2d::{Coord, Size};
use line_2d::LineSegment;
use rgb24::Rgb24;
use serde::{Deserialize, Serialize};
use std::mem;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Clone, Copy)]
struct WallId(RawId);
#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct CharacterId(RawId);

impl IdTrait for WallId {
    fn from_raw(raw_id: RawId) -> Self {
        Self(raw_id)
    }
    fn to_raw(self) -> RawId {
        self.0
    }
}

impl IdTrait for CharacterId {
    fn from_raw(raw_id: RawId) -> Self {
        Self(raw_id)
    }
    fn to_raw(self) -> RawId {
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

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum WallTile {
    Plain,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Wall {
    tile: WallTile,
}

impl Wall {
    pub fn tile(&self) -> WallTile {
        self.tile
    }
    pub fn is_solid(&self) -> bool {
        true
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum ProjectileTile {
    Bullet,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectileData {
    tile: ProjectileTile,
}

impl ProjectileData {
    pub fn tile(&self) -> ProjectileTile {
        self.tile
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct DoubleBuffer<T> {
    current: Vec<T>,
    next: Vec<T>,
}

impl<T> DoubleBuffer<T> {
    fn new() -> Self {
        Self {
            current: Vec::new(),
            next: Vec::new(),
        }
    }
}

pub struct BlendRgb24 {
    rgb24: Rgb24,
    blend_factor: u8,
}

impl BlendRgb24 {
    pub fn blend(&self, rgb24: Rgb24) -> Rgb24 {
        rgb24.linear_interpolate(self.rgb24, self.blend_factor)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ColouredGasGrid {
    gas_grid: GasGrid,
    from_rgb24: Rgb24,
    to_rgb24: Rgb24,
}

impl ColouredGasGrid {
    fn blend_grid_iter<'a>(&'a self) -> impl 'a + Iterator<Item = BlendRgb24> {
        self.gas_grid.iter().map(move |intensity| {
            let rgb24 = self.to_rgb24.linear_interpolate(self.from_rgb24, intensity);
            BlendRgb24 {
                rgb24,
                blend_factor: intensity,
            }
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Gas {
    bullet_trails: ColouredGasGrid,
}

impl Gas {
    fn new(size: Size) -> Self {
        let bullet_trails = ColouredGasGrid {
            gas_grid: GasGrid::new(
                size,
                GasSpec {
                    fade: GasRatio::new(1, 2),
                    spread: GasRatio::new(1, 2),
                },
            ),
            from_rgb24: Rgb24::new(255, 255, 255),
            to_rgb24: Rgb24::new(0, 0, 0),
        };
        Self { bullet_trails }
    }
    fn tick(&mut self, walls: &EntityGrid<WallId, Wall>) {
        self.bullet_trails.gas_grid.tick(|coord| {
            if let Some(cell) = walls.get_entity_by_coord(coord).unwrap() {
                !cell.entity.is_solid()
            } else {
                true
            }
        })
    }
    fn has_pending_animations(&self) -> bool {
        self.bullet_trails.gas_grid.is_empty()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct World {
    size: Size,
    walls: EntityGrid<WallId, Wall>,
    characters: EntityGrid<CharacterId, Character>,
    realtime_projectiles: DoubleBuffer<Projectile<ProjectileData>>,
    gas: Gas,
}

impl World {
    pub fn new(size: Size) -> Self {
        let walls = EntityGrid::new(size);
        let characters = EntityGrid::new(size);
        let realtime_projectiles = DoubleBuffer::new();
        let gas = Gas::new(size);
        Self {
            size,
            walls,
            characters,
            realtime_projectiles,
            gas,
        }
    }
    pub fn size(&self) -> Size {
        self.size
    }
    pub fn spawn_player(&mut self, coord: Coord) -> CharacterId {
        self.characters
            .spawn_entity(
                coord,
                Character {
                    tile: CharacterTile::Player,
                },
            )
            .unwrap()
    }
    pub fn spawn_wall(&mut self, coord: Coord) {
        self.walls.spawn_entity(coord, Wall { tile: WallTile::Plain }).unwrap();
    }
    pub fn character_coord(&self, character_id: CharacterId) -> Coord {
        self.characters.get_coord_by_id(character_id).unwrap()
    }
    pub fn character_walk_in_direction(&mut self, character_id: CharacterId, direction: CardinalDirection) {
        let select = self
            .characters
            .select_entity_to_move_to_empty_cell(character_id)
            .unwrap();
        let destination_coord = select.coord() + direction.coord();
        if !destination_coord.is_valid(self.size) {
            return;
        }
        if let Some(EntityByCoord { id: _, entity: wall }) = self.walls.get_entity_by_coord(destination_coord).unwrap()
        {
            if wall.is_solid() {
                return;
            }
        }
        let staged_move = select.stage_move_entity_to_empty_cell(destination_coord).unwrap();
        staged_move.commit();
    }
    pub fn character_fire_bullet(&mut self, character_id: CharacterId, target_coord: Coord) {
        let source_coord = self.characters.get_coord_by_id(character_id).unwrap();
        let path = LineSegment::new(source_coord, target_coord);
        let step_duration = Duration::from_millis(10);
        let value = ProjectileData {
            tile: ProjectileTile::Bullet,
        };
        let projectile = Projectile::new(path, step_duration, value);
        self.realtime_projectiles.current.push(projectile);
    }
    pub fn has_pending_animations(&self) -> bool {
        !self.realtime_projectiles.current.is_empty() || self.gas.has_pending_animations()
    }
    pub fn animation_tick(&mut self) {
        self.tick_realtime_projectiles();
        self.gas.tick(&self.walls);
    }
    fn tick_realtime_projectiles(&mut self) {
        for mut projectile in self.realtime_projectiles.current.drain(..) {
            let mut projectile_continue = true;
            for step in projectile.frame_iter() {
                let coord = match step {
                    Step::MoveTo(coord) => {
                        if let Ok(maybe_wall) = self.walls.get_entity_by_coord(coord) {
                            if let Some(wall) = maybe_wall {
                                if wall.entity.is_solid() {
                                    projectile_continue = false;
                                    break;
                                }
                            }
                        } else {
                            projectile_continue = false;
                            break;
                        }
                        coord
                    }
                    Step::AtDestination => {
                        projectile_continue = false;
                        break;
                    }
                };
                self.gas.bullet_trails.gas_grid.register(coord);
            }
            if projectile_continue {
                self.realtime_projectiles.next.push(projectile);
            }
        }
        mem::swap(
            &mut self.realtime_projectiles.current,
            &mut self.realtime_projectiles.next,
        );
    }
    pub fn to_render(&self) -> ToRender {
        ToRender { world: self }
    }
}

pub struct ToRender<'a> {
    world: &'a World,
}

impl<'a> ToRender<'a> {
    pub fn grid_enumerate(&self) -> impl Iterator<Item = (Coord, ToRenderCell)> {
        self.world
            .characters
            .grid_enumerate()
            .zip(self.world.walls.grid_iter())
            .map(|((coord, maybe_character_by_coord), maybe_wall_by_coord)| {
                (
                    coord,
                    ToRenderCell {
                        character: maybe_character_by_coord
                            .as_ref()
                            .map(|entity_by_coord| entity_by_coord.entity),
                        wall: maybe_wall_by_coord
                            .as_ref()
                            .map(|entity_by_coord| entity_by_coord.entity),
                    },
                )
            })
    }
    pub fn realtime_projectiles(&self) -> impl Iterator<Item = &Projectile<ProjectileData>> {
        self.world.realtime_projectiles.current.iter()
    }
    pub fn bullet_trail_blend_grid_iter(&self) -> impl 'a + Iterator<Item = BlendRgb24> {
        self.world.gas.bullet_trails.blend_grid_iter()
    }
}

pub struct ToRenderCell<'a> {
    pub character: Option<&'a Character>,
    pub wall: Option<&'a Wall>,
}
