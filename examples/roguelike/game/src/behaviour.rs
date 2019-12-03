use crate::visibility::Visibility;
use crate::Input;
use crate::World;
use ecs::Entity;
use grid_2d::{Coord, Grid, Size};
use grid_search_cardinal_best::{BestSearch, Context as BestSearchContext, Depth};
use grid_search_cardinal_point_to_point::{expand, Context as PointToPointSearchContext, NoPath, PointToPointSearch};
use serde::{Deserialize, Serialize};
use shadowcast::{vision_distance, Context as ShadowcastContext};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LastSeenCell {
    count: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LastSeenGrid {
    count: u64,
    last_seen: Grid<LastSeenCell>,
}

struct PlayerVisible(Coord);

impl LastSeenGrid {
    fn new(size: Size) -> Self {
        Self {
            count: 1,
            last_seen: Grid::new_fn(size, |_| LastSeenCell { count: 0 }),
        }
    }

    fn update(
        &mut self,
        eye: Coord,
        vision_distance: vision_distance::Circle,
        world: &World,
        player: Entity,
        shadowcast: &mut ShadowcastContext<u8>,
    ) -> Option<PlayerVisible> {
        let player_coord = world.entity_coord(player);
        let mut player_visible = None;
        self.count += 1;
        shadowcast.for_each_visible(
            eye,
            &Visibility,
            world,
            vision_distance,
            255,
            |cell_coord, _visible_directions, _visibility| {
                if cell_coord == player_coord {
                    player_visible = Some(PlayerVisible(player_coord));
                }
                if let Some(cell) = self.last_seen.get_mut(cell_coord) {
                    cell.count = self.count;
                }
            },
        );
        player_visible
    }
}

#[derive(Serialize, Deserialize)]
pub struct BehaviourContext {
    best_search_context: BestSearchContext,
    point_to_point_search_context: PointToPointSearchContext,
}

impl BehaviourContext {
    pub fn new(size: Size) -> Self {
        Self {
            best_search_context: BestSearchContext::new(size),
            point_to_point_search_context: PointToPointSearchContext::new(size),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Agent {
    last_seen_grid: LastSeenGrid,
    vision_distance: vision_distance::Circle,
    behaviour: Behaviour,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum Behaviour {
    Wander,
    Chase { last_seen_player_coord: Coord },
}

struct Wander<'a> {
    world: &'a World,
    last_seen_grid: &'a LastSeenGrid,
    min_last_seen_coord: Option<Coord>,
    min_last_seen_count: u64,
}

impl<'a> BestSearch for Wander<'a> {
    fn is_at_max_depth(&self, _depth: Depth) -> bool {
        false
    }
    fn can_enter_updating_best(&mut self, coord: Coord) -> bool {
        if !self.world.contains_wall(coord) {
            let last_seen_count = self.last_seen_grid.last_seen.get_checked(coord).count;
            if last_seen_count < self.min_last_seen_count {
                self.min_last_seen_count = last_seen_count;
                self.min_last_seen_coord = Some(coord);
            }
            true
        } else {
            false
        }
    }
    fn best_coord(&self) -> Option<Coord> {
        self.min_last_seen_coord
    }
}

struct Attack<'a> {
    world: &'a World,
}

impl<'a> PointToPointSearch for Attack<'a> {
    fn can_enter(&self, coord: Coord) -> bool {
        !self.world.contains_wall(coord)
    }
}

impl Agent {
    pub fn new(size: Size) -> Self {
        Self {
            last_seen_grid: LastSeenGrid::new(size),
            vision_distance: vision_distance::Circle::new_squared(40),
            behaviour: Behaviour::Wander,
        }
    }
    pub fn act(
        &mut self,
        coord: Coord,
        world: &World,
        player: Entity,
        behaviour_context: &mut BehaviourContext,
        shadowcast_context: &mut ShadowcastContext<u8>,
    ) -> Option<Input> {
        self.behaviour = if let Some(PlayerVisible(player_coord)) =
            self.last_seen_grid
                .update(coord, self.vision_distance, world, player, shadowcast_context)
        {
            Behaviour::Chase {
                last_seen_player_coord: player_coord,
            }
        } else {
            match self.behaviour {
                Behaviour::Chase { last_seen_player_coord } => {
                    if last_seen_player_coord == coord {
                        Behaviour::Wander
                    } else {
                        Behaviour::Chase { last_seen_player_coord }
                    }
                }
                Behaviour::Wander => Behaviour::Wander,
            }
        };
        match self.behaviour {
            Behaviour::Wander => {
                if let Some(cardinal_direction) = behaviour_context.best_search_context.best_search_first(
                    Wander {
                        world,
                        last_seen_grid: &self.last_seen_grid,
                        min_last_seen_coord: None,
                        min_last_seen_count: self.last_seen_grid.last_seen.get_checked(coord).count,
                    },
                    coord,
                ) {
                    Some(Input::Walk(cardinal_direction))
                } else {
                    None
                }
            }
            Behaviour::Chase { last_seen_player_coord } => {
                let result = behaviour_context
                    .point_to_point_search_context
                    .point_to_point_search_first(expand::JumpPoint, Attack { world }, coord, last_seen_player_coord);
                match result {
                    Err(NoPath) | Ok(None) => {
                        self.behaviour = Behaviour::Wander;
                        None
                    }
                    Ok(Some(cardinal_direction)) => Some(Input::Walk(cardinal_direction)),
                }
            }
        }
    }
}
