use crate::world::World;
use grid_2d::{Coord, Grid, Size};
use rgb24::Rgb24;
use serde::{Deserialize, Serialize};
use shadowcast::{vision_distance, DirectionBitmap, InputGrid, ShadowcastContext};

const VISION_DISTANCE_SQUARED: u32 = 150;
const VISION_DISTANCE: vision_distance::Circle = vision_distance::Circle::new_squared(VISION_DISTANCE_SQUARED);

struct Visibility;

impl InputGrid for Visibility {
    type Grid = World;
    type Opacity = u8;
    fn size(&self, world: &Self::Grid) -> Size {
        world.size()
    }
    fn get_opacity(&self, world: &Self::Grid, coord: Coord) -> Self::Opacity {
        world.opacity(coord)
    }
}

#[derive(Serialize, Deserialize)]
struct Cell {
    last_seen: u64,
    last_lit: u64,
    visible_directions: DirectionBitmap,
    light_colour: Rgb24,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            last_seen: 0,
            last_lit: 0,
            visible_directions: DirectionBitmap::empty(),
            light_colour: Rgb24::new(0, 0, 0),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct VisibilityGrid {
    grid: Grid<Cell>,
    count: u64,
    #[serde(skip)]
    shadowcast_context: ShadowcastContext<u8>,
}

impl VisibilityGrid {
    pub fn new(size: Size) -> Self {
        Self {
            grid: Grid::new_default(size),
            count: 0,
            shadowcast_context: ShadowcastContext::default(),
        }
    }
    pub fn is_visible(&self, coord: Coord) -> bool {
        if let Some(cell) = self.grid.get(coord) {
            cell.last_seen == self.count
        } else {
            false
        }
    }
    pub fn update(&mut self, player_coord: Coord, world: &World) {
        self.count += 1;
        let count = self.count;
        let grid = &mut self.grid;
        self.shadowcast_context.for_each_visible(
            player_coord,
            &Visibility,
            world,
            VISION_DISTANCE,
            255,
            |coord, visible_directions, _visibility| {
                let cell = grid.get_checked_mut(coord);
                cell.last_seen = count;
                cell.visible_directions = visible_directions;
            },
        );
    }
}
