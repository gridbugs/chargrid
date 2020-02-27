use crate::behaviour::Agent;
use crate::{
    world::EntityData,
    world::{Layer, Location},
    World,
};
use ecs::{ComponentTable, Entity};
use grid_2d::{Coord, Size};
use procgen::{LightType, Spaceship, SpaceshipCell, SpaceshipSpec};
use rand::{seq::IteratorRandom, Rng};
use rgb24::Rgb24;

pub struct Terrain {
    pub world: World,
    pub player: Entity,
    pub agents: ComponentTable<Agent>,
}

#[allow(dead_code)]
pub fn from_str(s: &str, player_data: EntityData) -> Terrain {
    let rows = s.split('\n').filter(|s| !s.is_empty()).collect::<Vec<_>>();
    let size = Size::new_u16(rows[0].len() as u16, rows.len() as u16);
    let mut world = World::new(size);
    let mut agents = ComponentTable::default();
    let mut player_data = Some(player_data);
    let mut player = None;
    for (y, row) in rows.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            if ch.is_control() {
                continue;
            }
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
                '=' => {
                    world.spawn_floor(coord);
                    world.spawn_window(coord);
                }
                '+' => {
                    world.spawn_floor(coord);
                    world.spawn_door(coord);
                }
                '%' => {
                    world.spawn_star(coord);
                    world.spawn_space(coord);
                }
                ' ' => {
                    world.spawn_space(coord);
                }
                '@' => {
                    world.spawn_floor(coord);
                    let location = Location {
                        coord,
                        layer: Some(Layer::Character),
                    };
                    player = Some(world.insert_entity_data(location, player_data.take().unwrap()));
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
                _ => log::warn!("unexpected char in terrain: {} ({})", ch.escape_unicode(), ch),
            }
        }
    }
    let player = player.expect("didn't create player");
    Terrain { world, player, agents }
}

pub fn spaceship<R: Rng>(spec: SpaceshipSpec, player_data: EntityData, rng: &mut R) -> Terrain {
    let mut world = World::new(spec.size);
    let mut agents = ComponentTable::default();
    let spaceship = Spaceship::generate(spec, rng);
    let mut npc_candidates = Vec::new();
    for (coord, cell) in spaceship.map.enumerate() {
        match cell {
            SpaceshipCell::Wall => {
                world.spawn_wall(coord);
            }
            SpaceshipCell::Floor => {
                world.spawn_floor(coord);
                npc_candidates.push(coord);
            }
            SpaceshipCell::Space => {
                world.spawn_space(coord);
            }
            SpaceshipCell::Door => {
                world.spawn_floor(coord);
                world.spawn_door(coord);
            }
            SpaceshipCell::Window => {
                world.spawn_window(coord);
            }
            SpaceshipCell::Star => {
                world.spawn_star(coord);
                world.spawn_space(coord);
            }
        }
    }
    for light in spaceship.lights.iter() {
        match light.typ {
            LightType::Working => {
                world.spawn_light(light.coord, Rgb24::new(187, 187, 187));
            }
            LightType::Emergency => {
                world.spawn_light(light.coord, Rgb24::new(187, 0, 0));
            }
            LightType::Flickering => {
                world.spawn_flickering_light(light.coord, Rgb24::new(187, 187, 187));
            }
            LightType::Broken => (),
        }
    }
    world.spawn_stairs(spaceship.exit);
    //world.spawn_stairs(spaceship.player_spawn + Coord::new(1, 0));
    let player_location = Location {
        coord: spaceship.player_spawn,
        layer: Some(Layer::Character),
    };
    let player = world.insert_entity_data(player_location, player_data);
    for coord in npc_candidates
        .into_iter()
        .filter(|&coord| !world.is_character_at_coord(coord))
        .choose_multiple(rng, 20)
    {
        let entity = world.spawn_former_human(coord);
        agents.insert(entity, Agent::new(spec.size));
    }
    Terrain { world, player, agents }
}
