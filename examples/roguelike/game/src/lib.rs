pub use direction::CardinalDirection;
pub use grid_2d::{Coord, Grid, Size};
use rand::{Rng, SeedableRng};
use rand_isaac::Isaac64Rng;
use rgb24::Rgb24;
use serde::{Deserialize, Serialize};
use std::time::Duration;

mod rational;
mod visibility;
mod world;

pub use visibility::{CellVisibility, VisibilityGrid};
use world::{Entity, World};
pub use world::{Layer, Tile, ToRenderEntity};

#[derive(Clone, Copy)]
pub enum Input {
    Walk(CardinalDirection),
    Fire(Coord),
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    world: World,
    visibility_grid: VisibilityGrid,
    player: Entity,
    rng: Isaac64Rng,
    frame_count: u64,
}

impl Game {
    pub fn new<R: Rng>(rng: &mut R) -> Self {
        let s = include_str!("terrain.txt");
        let rows = s.split('\n').filter(|s| !s.is_empty()).collect::<Vec<_>>();
        let size = Size::new_u16(rows[0].len() as u16, rows.len() as u16);
        let mut world = World::new(size);
        let mut player = None;
        for (y, row) in rows.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                let coord = Coord::new(x as i32, y as i32);
                match ch {
                    '.' => {
                        world.spawn_floor(coord);
                    }
                    '*' => {
                        world.spawn_floor(coord);
                        world.spawn_light(coord, Rgb24::new(187, 187, 187));
                    }
                    ',' => {
                        world.spawn_carpet(coord);
                    }
                    '#' => {
                        world.spawn_floor(coord);
                        world.spawn_wall(coord);
                    }
                    '@' => {
                        world.spawn_floor(coord);
                        player = Some(world.spawn_player(coord));
                    }
                    _ => panic!("unexpected char: {}", ch),
                }
            }
        }
        let mut game = Self {
            world,
            visibility_grid: VisibilityGrid::new(size),
            player: player.expect("didn't create player"),
            rng: Isaac64Rng::seed_from_u64(rng.gen()),
            frame_count: 0,
        };
        game.update_visibility();
        game
    }
    pub fn is_gameplay_blocked(&self) -> bool {
        self.world.is_gameplay_blocked()
    }
    fn update_visibility(&mut self) {
        let player_coord = self.world.entity_coord(self.player);
        self.visibility_grid.update(player_coord, &self.world);
    }
    pub fn handle_input(&mut self, input: Input) {
        if !self.is_gameplay_blocked() {
            match input {
                Input::Walk(direction) => self.world.character_walk_in_direction(self.player, direction),
                Input::Fire(coord) => self.world.character_fire_bullet(self.player, coord),
            }
        }
        self.update_visibility();
    }
    pub fn handle_tick(&mut self, _since_last_tick: Duration) {
        self.update_visibility();
        self.world.animation_tick(&mut self.rng);
        self.frame_count += 1;
    }
    pub fn player_coord(&self) -> Coord {
        self.world.entity_coord(self.player)
    }
    pub fn world_size(&self) -> Size {
        self.world.size()
    }
    pub fn to_render_entities<'a>(&'a self) -> impl 'a + Iterator<Item = ToRenderEntity> {
        self.world.to_render_entities()
    }
    pub fn visibility_grid(&self) -> &VisibilityGrid {
        &self.visibility_grid
    }
}
