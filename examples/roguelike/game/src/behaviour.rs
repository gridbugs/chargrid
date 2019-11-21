use crate::visibility::Visibility;
use crate::Input;
use crate::World;
use direction::{CardinalDirection, CardinalDirections};
use grid_2d::{Coord, Grid, Size};
use serde::{Deserialize, Serialize};
use shadowcast::{vision_distance, ShadowcastContext};
use std::collections::VecDeque;

#[derive(Serialize, Deserialize)]
struct SeenCell {
    count: u64,
    in_direction: Option<CardinalDirection>,
}

struct UnseenCell<'a> {
    count: u64,
    in_direction: CardinalDirection,
    coord: Coord,
    seen_cell: &'a mut SeenCell,
    queue: &'a mut VecDeque<Coord>,
}

impl<'a> UnseenCell<'a> {
    fn see(self) {
        self.seen_cell.count = self.count;
        self.seen_cell.in_direction = Some(self.in_direction);
        self.queue.push_back(self.coord);
    }
}

#[derive(Serialize, Deserialize)]
struct SearchContext {
    count: u64,
    seen_set: Grid<SeenCell>,
    queue: VecDeque<Coord>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PathNode {
    to_coord: Coord,
    in_direction: CardinalDirection,
}

#[derive(Debug)]
enum BuildPathError {
    EndNotVisited,
    EndOutOfBounds,
}

impl SearchContext {
    fn new(size: Size) -> Self {
        Self {
            count: 1,
            seen_set: Grid::new_fn(size, |_| SeenCell {
                count: 0,
                in_direction: None,
            }),
            queue: VecDeque::new(),
        }
    }
    fn init(&mut self, coord: Coord) {
        self.count += 1;
        self.queue.clear();
        self.queue.push_back(coord);
        let cell = self.seen_set.get_checked_mut(coord);
        cell.count = self.count;
        cell.in_direction = None;
    }
    fn dequeue(&mut self) -> Option<Coord> {
        self.queue.pop_front()
    }
    fn unseen_cell(&mut self, coord: Coord, in_direction: CardinalDirection) -> Option<UnseenCell> {
        if let Some(seen_cell) = self.seen_set.get_mut(coord) {
            if seen_cell.count != self.count {
                return Some(UnseenCell {
                    count: self.count,
                    in_direction,
                    coord,
                    seen_cell,
                    queue: &mut self.queue,
                });
            }
        }
        None
    }
    fn build_path_to(&self, end: Coord, path: &mut Vec<PathNode>) -> Result<(), BuildPathError> {
        let mut cell = self.seen_set.get(end).ok_or(BuildPathError::EndOutOfBounds)?;
        let mut coord = end;
        if cell.count != self.count {
            return Err(BuildPathError::EndNotVisited);
        }
        path.clear();
        while let Some(in_direction) = cell.in_direction {
            let path_node = PathNode {
                to_coord: coord,
                in_direction,
            };
            path.push(path_node);
            coord = coord - in_direction.coord();
            cell = self.seen_set.get_checked(coord);
            debug_assert_eq!(cell.count, self.count);
        }
        path.reverse();
        Ok(())
    }
}

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

    fn path_to_oldest_cell(
        &self,
        start: Coord,
        world: &World,
        path: &mut Vec<PathNode>,
        search_context: &mut SearchContext,
    ) {
        let mut best_coord = start;
        let mut min_last_seen = self.last_seen.get_checked(start).count;
        search_context.init(start);
        while let Some(visited_coord) = search_context.dequeue() {
            for direction in CardinalDirections {
                let seen_coord = visited_coord + direction.coord();
                if let Some(unseen_cell) = search_context.unseen_cell(seen_coord, direction) {
                    if !world.contains_wall(seen_coord) {
                        unseen_cell.see();
                        let last_seen = self.last_seen.get_checked(seen_coord).count;
                        if last_seen < min_last_seen {
                            min_last_seen = last_seen;
                            best_coord = seen_coord;
                        }
                    }
                }
            }
        }
        search_context.build_path_to(best_coord, path).unwrap();
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
    path: Vec<PathNode>,
}

impl Agent {
    pub fn new(size: Size) -> Self {
        Self {
            last_seen_grid: LastSeenGrid::new(size),
            vision_distance: vision_distance::Circle::new_squared(40),
            path: Vec::new(),
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
        self.last_seen_grid
            .path_to_oldest_cell(coord, world, &mut self.path, &mut behaviour_context.search_context);
        self.path.first().map(|path_node| Input::Walk(path_node.in_direction))
    }
}
