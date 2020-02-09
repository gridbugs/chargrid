use crate::visibility::Visibility;
use crate::world::Disposition;
use crate::world::World;
use crate::Input;
use ecs::Entity;
use grid_2d::{Coord, Grid, Size};
use grid_search_cardinal::{
    best::{BestSearch, Context as BestSearchContext, Depth},
    distance_map::{
        Distance, DistanceMap, PopulateContext as DistanceMapPopulateContext, SearchContext as DistanceMapSearchContext,
    },
    point_to_point::{expand, Context as PointToPointSearchContext, NoPath},
    CanEnter,
};
use line_2d::LineSegment;
use serde::{Deserialize, Serialize};
use shadowcast::{vision_distance, Context as ShadowcastContext, VisionDistance};

const FLEE_DISTANCE: Distance = 5;

fn has_line_of_sight(eye: Coord, dest: Coord, world: &World, vision_distance: vision_distance::Circle) -> bool {
    for coord in LineSegment::new(eye, dest).iter() {
        let eye_to_coord = coord - eye;
        if !vision_distance.in_range(eye_to_coord) {
            return false;
        }
        if world.is_wall_at_coord(coord) {
            return false;
        }
    }
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LastSeenCell {
    count: u64,
    avoid_until: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LastSeenGrid {
    count: u64,
    last_seen: Grid<LastSeenCell>,
}

#[derive(Clone, Copy, Debug)]
struct CanSeePlayer;

impl LastSeenGrid {
    fn new(size: Size) -> Self {
        Self {
            count: 1,
            last_seen: Grid::new_fn(size, |_| LastSeenCell {
                count: 0,
                avoid_until: 0,
            }),
        }
    }

    fn update(
        &mut self,
        eye: Coord,
        vision_distance: vision_distance::Circle,
        world: &World,
        can_see_player: Option<CanSeePlayer>,
        behaviour_context: &BehaviourContext,
        shadowcast: &mut ShadowcastContext<u8>,
    ) {
        self.count += 1;
        let distance_map_to_player = &behaviour_context.player_approach;
        shadowcast.for_each_visible(
            eye,
            &Visibility,
            world,
            vision_distance,
            255,
            |cell_coord, _visible_directions, _visibility| {
                if let Some(cell) = self.last_seen.get_mut(cell_coord) {
                    cell.count = self.count;
                    if let Some(CanSeePlayer) = can_see_player {
                        if let Some(distance_to_player) = distance_map_to_player.distance(cell_coord) {
                            if distance_to_player < FLEE_DISTANCE {
                                cell.avoid_until = self.count + 20;
                            }
                        }
                    }
                }
            },
        );
    }
}

#[derive(Serialize, Deserialize)]
pub struct BehaviourContext {
    best_search_context: BestSearchContext,
    point_to_point_search_context: PointToPointSearchContext,
    distance_map_populate_context: DistanceMapPopulateContext,
    distance_map_search_context: DistanceMapSearchContext,
    player_approach: DistanceMap,
    player_flee: DistanceMap,
}

impl BehaviourContext {
    pub fn new(size: Size) -> Self {
        Self {
            best_search_context: BestSearchContext::new(size),
            point_to_point_search_context: PointToPointSearchContext::new(size),
            distance_map_populate_context: DistanceMapPopulateContext::default(),
            distance_map_search_context: DistanceMapSearchContext::new(size),
            player_approach: DistanceMap::new(size),
            player_flee: DistanceMap::new(size),
        }
    }
    pub fn update(&mut self, player: Entity, world: &World) {
        if let Some(player_coord) = world.entity_coord(player) {
            let can_enter = WorldCanEnterIgnoreCharacters { world };
            self.distance_map_populate_context.add(player_coord);
            self.distance_map_populate_context
                .populate_approach(&can_enter, 20, &mut self.player_approach);
            self.distance_map_populate_context.add(player_coord);
            self.distance_map_populate_context
                .populate_flee(&can_enter, 20, &mut self.player_flee);
        } else {
            self.player_approach.clear();
            self.player_flee.clear();
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
    Wander {
        avoid: bool,
    },
    Chase {
        last_seen_player_coord: Coord,
        accurate: bool,
    },
    Flee,
}

struct Wander<'a> {
    world: &'a World,
    last_seen_grid: &'a LastSeenGrid,
    min_last_seen_coord: Option<Coord>,
    min_last_seen_count: u64,
    entity: Entity,
    avoid: bool,
}

impl<'a> BestSearch for Wander<'a> {
    fn is_at_max_depth(&self, _depth: Depth) -> bool {
        false
    }
    fn can_enter_updating_best(&mut self, coord: Coord) -> bool {
        if !self.world.is_wall_at_coord(coord) {
            if let Some(entity) = self.world.get_character_at_coord(coord) {
                if entity != self.entity {
                    return false;
                }
            }
            let last_seen_cell = self.last_seen_grid.last_seen.get_checked(coord);
            if self.avoid && last_seen_cell.avoid_until > self.min_last_seen_count {
                return false;
            }
            let last_seen_count = last_seen_cell.count;
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

struct WorldCanEnterIgnoreCharacters<'a> {
    world: &'a World,
}

impl<'a> CanEnter for WorldCanEnterIgnoreCharacters<'a> {
    fn can_enter(&self, coord: Coord) -> bool {
        !self.world.is_wall_at_coord(coord)
    }
}

struct WorldCanEnterAvoidNpcs<'a> {
    world: &'a World,
}

impl<'a> CanEnter for WorldCanEnterAvoidNpcs<'a> {
    fn can_enter(&self, coord: Coord) -> bool {
        !self.world.is_wall_at_coord(coord) && !self.world.is_npc_at_coord(coord)
    }
}

impl Agent {
    pub fn new(size: Size) -> Self {
        Self {
            last_seen_grid: LastSeenGrid::new(size),
            vision_distance: vision_distance::Circle::new_squared(40),
            behaviour: Behaviour::Wander { avoid: true },
        }
    }
    pub fn act(
        &mut self,
        entity: Entity,
        world: &World,
        player: Entity,
        behaviour_context: &mut BehaviourContext,
        shadowcast_context: &mut ShadowcastContext<u8>,
    ) -> Option<Input> {
        let coord = world.entity_coord(entity)?;
        let npc = world.entity_npc(entity);
        self.behaviour = if let Some(player_coord) = world.entity_coord(player) {
            let can_see_player = if has_line_of_sight(coord, player_coord, world, self.vision_distance) {
                Some(CanSeePlayer)
            } else {
                None
            };
            self.last_seen_grid.update(
                coord,
                self.vision_distance,
                world,
                can_see_player,
                behaviour_context,
                shadowcast_context,
            );
            if let Some(CanSeePlayer) = can_see_player {
                match npc.disposition {
                    Disposition::Hostile => Behaviour::Chase {
                        last_seen_player_coord: player_coord,
                        accurate: true,
                    },
                    Disposition::Afraid => {
                        if behaviour_context.player_approach.distance(coord).unwrap() < FLEE_DISTANCE {
                            Behaviour::Flee
                        } else {
                            Behaviour::Wander { avoid: true }
                        }
                    }
                }
            } else {
                match self.behaviour {
                    Behaviour::Chase {
                        last_seen_player_coord, ..
                    } => {
                        if last_seen_player_coord == coord {
                            // walk up to where the player was last seen, then go back to wandering
                            Behaviour::Wander { avoid: true }
                        } else {
                            Behaviour::Chase {
                                last_seen_player_coord,
                                accurate: last_seen_player_coord == player_coord,
                            }
                        }
                    }
                    Behaviour::Wander { avoid } => Behaviour::Wander { avoid },
                    Behaviour::Flee => {
                        // stop fleeing the player if you can't see them
                        Behaviour::Wander { avoid: true }
                    }
                }
            }
        } else {
            self.last_seen_grid.update(
                coord,
                self.vision_distance,
                world,
                None,
                behaviour_context,
                shadowcast_context,
            );
            Behaviour::Wander { avoid: false }
        };
        match self.behaviour {
            Behaviour::Wander { avoid } => {
                if let Some(cardinal_direction) = behaviour_context.best_search_context.best_search_first(
                    Wander {
                        world,
                        last_seen_grid: &self.last_seen_grid,
                        min_last_seen_coord: None,
                        min_last_seen_count: self.last_seen_grid.last_seen.get_checked(coord).count,
                        entity,
                        avoid,
                    },
                    coord,
                ) {
                    Some(Input::Walk(cardinal_direction))
                } else {
                    None
                }
            }
            Behaviour::Flee => {
                let maybe_cardinal_direction = behaviour_context.distance_map_search_context.search_first(
                    &WorldCanEnterAvoidNpcs { world },
                    coord,
                    5,
                    &behaviour_context.player_flee,
                );
                match maybe_cardinal_direction {
                    None => {
                        self.behaviour = Behaviour::Wander { avoid: true };
                        None
                    }
                    Some(cardinal_direction) => Some(Input::Walk(cardinal_direction)),
                }
            }
            Behaviour::Chase {
                last_seen_player_coord,
                accurate,
            } => {
                if accurate {
                    let maybe_cardinal_direction = behaviour_context.distance_map_search_context.search_first(
                        &WorldCanEnterAvoidNpcs { world },
                        coord,
                        5,
                        &behaviour_context.player_approach,
                    );
                    match maybe_cardinal_direction {
                        None => {
                            self.behaviour = Behaviour::Wander { avoid: true };
                            None
                        }
                        Some(cardinal_direction) => Some(Input::Walk(cardinal_direction)),
                    }
                } else {
                    let result = behaviour_context
                        .point_to_point_search_context
                        .point_to_point_search_first(
                            expand::JumpPoint,
                            &WorldCanEnterAvoidNpcs { world },
                            coord,
                            last_seen_player_coord,
                        );
                    match result {
                        Err(NoPath) | Ok(None) => {
                            self.behaviour = Behaviour::Wander { avoid: true };
                            None
                        }
                        Ok(Some(cardinal_direction)) => Some(Input::Walk(cardinal_direction)),
                    }
                }
            }
        }
    }
}
