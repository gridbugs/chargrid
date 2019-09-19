pub use direction::Direction;
pub use grid_2d::{Coord, Grid, Size};
use line_2d::LineSegment;
use rand::{Rng, SeedableRng};
use rand_isaac::Isaac64Rng;
use serde::{Deserialize, Serialize};
use std::time::Duration;

mod data;
mod particle;

use data::{Cell, GameData, Id};
use particle::*;

#[derive(Clone, Copy)]
pub enum Input {
    Move(Direction),
    Fire(Coord),
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    data: GameData,
    player_id: Id,
    particle_system: ParticleSystem,
    rng: Isaac64Rng,
}

impl Game {
    pub fn new<R: Rng>(rng: &mut R) -> Self {
        let s = include_str!("terrain.txt");
        let rows = s.split("\n").filter(|s| !s.is_empty()).collect::<Vec<_>>();
        let size = Size::new_u16(rows[0].len() as u16, rows.len() as u16);
        let mut data = GameData::new(size);
        let mut player_id = None;
        for (y, row) in rows.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                let coord = Coord::new(x as i32, y as i32);
                match ch {
                    '.' => (),
                    '#' => data.make_wall(coord),
                    '@' => {
                        player_id = Some(data.make_player(coord));
                    }
                    _ => panic!("unexpected char: {}", ch),
                }
            }
        }
        Self {
            data,
            player_id: player_id.expect("didn't create player"),
            particle_system: ParticleSystem::new(size),
            rng: Isaac64Rng::seed_from_u64(rng.gen()),
        }
    }
    pub fn has_animations(&self) -> bool {
        !self.particle_system.is_empty()
    }
    pub fn handle_input(&mut self, input: Input) {
        if !self.has_animations() {
            match input {
                Input::Move(direction) => self.data.move_character(self.player_id, direction),
                Input::Fire(coord) => {
                    let player_coord = self.player_coord();
                    if coord != player_coord {
                        let path = LineSegment::new(player_coord, coord);
                        let particle = Particle::new(path, Duration::from_millis(20));
                        self.particle_system.register(particle);
                    }
                }
            }
        }
    }
    pub fn handle_tick(&mut self, _since_last_tick: Duration) {
        self.particle_system.tick(&mut self.data);
    }
    pub fn grid(&self) -> &Grid<Cell> {
        self.data.grid()
    }
    pub fn particles(&self) -> &[Particle] {
        self.particle_system.particles()
    }
    pub fn trails_grid(&self) -> &Grid<TrailCell> {
        self.particle_system.trails_grid()
    }
    pub fn player_coord(&self) -> Coord {
        *self.data.coords().get(&self.player_id).unwrap()
    }
}
