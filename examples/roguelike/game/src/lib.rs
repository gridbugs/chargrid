pub use direction::CardinalDirection;
pub use grid_2d::{Coord, Grid, Size};
use rand::{Rng, SeedableRng};
use rand_isaac::Isaac64Rng;
use rgb24::Rgb24;
use serde::{Deserialize, Serialize};
use shadowcast::Context as ShadowcastContext;
use std::time::Duration;

mod behaviour;
mod rational;
mod visibility;
mod world;

use behaviour::{Agent, BehaviourContext};
use ecs::ComponentTable;
pub use visibility::{CellVisibility, Omniscient, VisibilityGrid};
use world::{Entity, World};
pub use world::{Layer, Tile, ToRenderEntity};

pub struct Config {
    pub omniscient: Option<Omniscient>,
}

/// Events which the game can report back to the io layer so it can
/// respond with a sound/visual effect.
#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum Event {
    Explosion(Coord),
}

#[derive(Clone, Copy, Debug)]
pub enum Input {
    Walk(CardinalDirection),
    Fire(Coord),
    Wait,
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    world: World,
    visibility_grid: VisibilityGrid,
    player: Entity,
    rng: Isaac64Rng,
    frame_count: u64,
    events: Vec<Event>,
    shadowcast_context: ShadowcastContext<u8>,
    behaviour_context: BehaviourContext,
    agents: ComponentTable<Agent>,
}

impl Game {
    pub fn new<R: Rng>(config: &Config, rng: &mut R) -> Self {
        let s = include_str!("terrain.txt");
        let rows = s.split('\n').filter(|s| !s.is_empty()).collect::<Vec<_>>();
        let size = Size::new_u16(rows[0].len() as u16, rows.len() as u16);
        let mut world = World::new(size);
        let mut agents = ComponentTable::default();
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
                    'f' => {
                        world.spawn_floor(coord);
                        let entity = world.spawn_former_human(coord);
                        agents.insert(entity, Agent::new(size));
                    }
                    'h' => {
                        world.spawn_floor(coord);
                        let entity = world.spawn_human(coord);
                        agents.insert(entity, Agent::new(size));
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
            events: Vec::new(),
            shadowcast_context: ShadowcastContext::default(),
            behaviour_context: BehaviourContext::new(size),
            agents,
        };
        game.update_behaviour();
        game.update_visibility(config);
        game
    }
    pub fn is_gameplay_blocked(&self) -> bool {
        self.world.is_gameplay_blocked()
    }
    pub fn update_visibility(&mut self, config: &Config) {
        let player_coord = self.world.entity_coord(self.player);
        self.visibility_grid.update(
            player_coord,
            &self.world,
            &mut self.shadowcast_context,
            config.omniscient,
        );
    }
    pub fn update_behaviour(&mut self) {
        self.behaviour_context.update(self.player, &self.world);
    }
    pub fn handle_input(&mut self, input: Input, config: &Config) {
        if !self.is_gameplay_blocked() {
            match input {
                Input::Walk(direction) => self.world.character_walk_in_direction(self.player, direction),
                Input::Fire(coord) => self.world.character_fire_bullet(self.player, coord),
                Input::Wait => (),
            }
        }
        self.update_visibility(config);
        self.update_behaviour();
        self.npc_turn();
    }
    pub fn handle_tick(&mut self, _since_last_tick: Duration, config: &Config) {
        self.events.clear();
        self.update_visibility(config);
        self.world.animation_tick(&mut self.events, &mut self.rng);
        self.frame_count += 1;
    }
    pub fn events(&self) -> impl '_ + Iterator<Item = Event> {
        self.events.iter().cloned()
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
    pub fn contains_wall(&self, coord: Coord) -> bool {
        self.world.contains_wall(coord)
    }
    pub fn npc_turn(&mut self) {
        for (entity, agent) in self.agents.iter_mut() {
            if let Some(input) = agent.act(
                entity,
                &self.world,
                self.player,
                &mut self.behaviour_context,
                &mut self.shadowcast_context,
            ) {
                match input {
                    Input::Walk(direction) => self.world.character_walk_in_direction(entity, direction),
                    Input::Fire(coord) => self.world.character_fire_bullet(entity, coord),
                    Input::Wait => (),
                }
            }
        }
    }
}
