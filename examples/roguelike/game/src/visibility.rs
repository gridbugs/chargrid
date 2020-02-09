use crate::world::World;
use grid_2d::{Coord, Grid, Size};
use rational::Rational;
use rgb24::Rgb24;
use serde::{Deserialize, Serialize};
use shadowcast::{vision_distance, Context as ShadowcastContext, DirectionBitmap, InputGrid};

const VISION_DISTANCE_SQUARED: u32 = 400;
const VISION_DISTANCE: vision_distance::Circle = vision_distance::Circle::new_squared(VISION_DISTANCE_SQUARED);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Light {
    pub colour: Rgb24,
    pub vision_distance: vision_distance::Circle,
    pub diminish: Rational,
}

pub struct Visibility;

impl InputGrid for Visibility {
    type Grid = World;
    type Opacity = u8;
    fn size(&self, world: &Self::Grid) -> Size {
        world.size()
    }
    fn get_opacity(&self, world: &Self::Grid, coord: Coord) -> Self::Opacity {
        world.get_opacity_at_coord(coord)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Omniscient;

#[derive(Serialize, Deserialize)]
struct VisibilityCell {
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

#[derive(Serialize, Deserialize)]
pub struct VisibilityGrid {
    grid: Grid<VisibilityCell>,
    count: u64,
}

pub enum CellVisibility {
    NotVisible,
    VisibleWithLightColour(Rgb24),
}

impl VisibilityGrid {
    pub fn new(size: Size) -> Self {
        Self {
            grid: Grid::new_default(size),
            count: 0,
        }
    }
    pub fn cell_visibility(&self, coord: Coord) -> CellVisibility {
        if let Some(cell) = self.grid.get(coord) {
            if cell.last_seen == self.count && cell.last_lit == self.count {
                CellVisibility::VisibleWithLightColour(cell.light_colour)
            } else {
                CellVisibility::NotVisible
            }
        } else {
            CellVisibility::NotVisible
        }
    }
    pub fn update(
        &mut self,
        player_coord: Coord,
        world: &World,
        shadowcast_context: &mut ShadowcastContext<u8>,
        omniscient: Option<Omniscient>,
    ) {
        self.count += 1;
        let count = self.count;
        let grid = &mut self.grid;
        if let Some(Omniscient) = omniscient {
            let size = world.size();
            for i in 0..size.y() {
                for j in 0..size.x() {
                    let coord = Coord::new(j as i32, i as i32);
                    let cell = grid.get_checked_mut(coord);
                    cell.last_seen = count;
                    cell.visible_directions = DirectionBitmap::all();
                    cell.last_lit = count;
                    cell.light_colour = Rgb24::new(255, 255, 255);
                }
            }
        } else {
            shadowcast_context.for_each_visible(
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
        for (light_coord, light) in world.all_lights_by_coord() {
            shadowcast_context.for_each_visible(
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
