pub use direction::Direction;
pub use grid_2d::{Coord, Grid, Size};
use line_2d::LineSegment;
use rand::{Rng, SeedableRng};
use rand_isaac::Isaac64Rng;
use serde::{Deserialize, Serialize};
use std::time::Duration;

mod animation;
mod circle;
mod data;
mod particle;

use data::{Cell, GameData, Id};

#[derive(Clone, Copy)]
pub enum Input {
    Move(Direction),
    Fire(Coord),
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    data: GameData,
    player_id: Id,
    animation_schedule: animation::Schedule,
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
            animation_schedule: animation::Schedule::new(),
            rng: Isaac64Rng::seed_from_u64(rng.gen()),
        }
    }
    pub fn has_animations(&self) -> bool {
        !self.animation_schedule.is_empty()
    }
    pub fn handle_input(&mut self, input: Input) {
        if self.animation_schedule.is_empty() {
            match input {
                Input::Move(direction) => self.data.move_character(self.player_id, direction),
                Input::Fire(coord) => {
                    let player_coord = self.player_coord();
                    if coord != player_coord {
                        let projectile = animation::Projectile::new(
                            LineSegment::new(player_coord, coord),
                            Duration::from_millis(20),
                            &mut self.data,
                        );
                        let explode_factory = animation::ExplodeFactory::new(10, &mut self.rng);
                        let animation = animation::AndThenCoord::new(Box::new(projectile), Box::new(explode_factory));
                        self.animation_schedule.register(Box::new(animation));
                    }
                }
            }
        }
    }
    pub fn handle_tick(&mut self, since_last_tick: Duration) {
        self.animation_schedule.tick(since_last_tick, &mut self.data);
    }
    pub fn grid(&self) -> &Grid<Cell> {
        self.data.grid()
    }
    pub fn player_coord(&self) -> Coord {
        *self.data.coords().get(&self.player_id).unwrap()
    }
}
