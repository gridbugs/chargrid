use crate::visibility::Visibility;
use crate::Input;
use crate::World;
use grid_2d::{Coord, Grid, Size};
use grid_search_cardinal_best::{BestSearch, Context as SearchContext, Depth};
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
        shadowcast: &mut ShadowcastContext<u8>,
    ) {
        self.count += 1;
        shadowcast.for_each_visible(
            eye,
            &Visibility,
            world,
            vision_distance,
            255,
            |cell_coord, _visible_directions, _visibility| {
                if let Some(cell) = self.last_seen.get_mut(cell_coord) {
                    cell.count = self.count;
                }
            },
        );
    }
}

#[derive(Serialize, Deserialize)]
pub struct BehaviourContext {
    search_context: SearchContext,
}

impl BehaviourContext {
    pub fn new(size: Size) -> Self {
        Self {
            search_context: SearchContext::new(size),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Agent {
    last_seen_grid: LastSeenGrid,
    vision_distance: vision_distance::Circle,
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

impl Agent {
    pub fn new(size: Size) -> Self {
        Self {
            last_seen_grid: LastSeenGrid::new(size),
            vision_distance: vision_distance::Circle::new_squared(40),
        }
    }
    pub fn act(
        &mut self,
        coord: Coord,
        world: &World,
        behaviour_context: &mut BehaviourContext,
        shadowcast_context: &mut ShadowcastContext<u8>,
    ) -> Option<Input> {
        self.last_seen_grid
            .update(coord, self.vision_distance, world, shadowcast_context);
        if let Some(cardinal_direction) = behaviour_context.search_context.best_search_first(
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
}
