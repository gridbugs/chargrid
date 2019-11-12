use crate::world::World;
use grid_2d::{Coord, Grid, Size};
use rgb24::Rgb24;
use serde::{Deserialize, Serialize};
use shadowcast::{vision_distance, DirectionBitmap, InputGrid, ShadowcastContext};

const VISION_DISTANCE_SQUARED: u32 = 150;
const VISION_DISTANCE: vision_distance::Circle = vision_distance::Circle::new_squared(VISION_DISTANCE_SQUARED);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rational {
    pub numerator: u32,
    pub denominator: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Light {
    pub colour: Rgb24,
    pub vision_distance: vision_distance::Circle,
    pub diminish: Rational,
}

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
pub struct VisibilityCell {
    last_seen: u64,
    last_lit: u64,
    visible_directions: DirectionBitmap,
    light_colour: Rgb24,
}

impl Default for VisibilityCell {
    fn default() -> Self {
        Self {
            last_seen: 0,
            last_lit: 0,
            visible_directions: DirectionBitmap::empty(),
            light_colour: Rgb24::new(0, 0, 0),
        }
    }
}

impl VisibilityCell {
    pub fn light_colour(&self) -> Rgb24 {
        self.light_colour
    }
}

#[derive(Serialize, Deserialize)]
pub struct VisibilityGrid {
    grid: Grid<VisibilityCell>,
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
    pub fn visible_cell(&self, coord: Coord) -> Option<&VisibilityCell> {
        if let Some(cell) = self.grid.get(coord) {
            if cell.last_seen == self.count {
                Some(cell)
            } else {
                None
            }
        } else {
            None
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
        for (light_coord, light) in world.lights() {
            self.shadowcast_context.for_each_visible(
                light_coord,
                &Visibility,
                world,
                light.vision_distance,
                255,
                |cell_coord, visible_directions, visibility| {
                    let cell = grid.get_checked_mut(cell_coord);
                    if cell.last_seen == count && !(visible_directions & cell.visible_directions).is_empty() {
                        if cell.last_lit != count {
                            cell.last_lit = count;
                            cell.light_colour = Rgb24::new(0, 0, 0);
                        }
                        let distance_squared = (light_coord - cell_coord).magnitude2();
                        let inverse_light_intensity =
                            (distance_squared * light.diminish.numerator) / light.diminish.denominator;
                        let light_colour = light.colour.scalar_div(inverse_light_intensity.max(1));
                        cell.light_colour = cell
                            .light_colour
                            .saturating_add(light_colour.normalised_scalar_mul(visibility));
                    }
                },
            );
        }
    }
}
