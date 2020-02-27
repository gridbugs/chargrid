use grid_2d::Grid;
use grid_2d::{Coord, Size};
use rand::{seq::SliceRandom, Rng};

mod hull;
use hull::generate_hull;

mod internal_walls;
use internal_walls::add_internal_walls;

mod connections;
use connections::add_connections;

mod stars;
use stars::add_stars;
pub use stars::StarCell as SpaceshipCell;

mod lights;
use lights::choose_lights;
pub use lights::{Light, LightType};

mod spawns;
use spawns::all_room_means;

pub struct Spaceship {
    pub lights: Vec<Light>,
    pub map: Grid<SpaceshipCell>,
    pub player_spawn: Coord,
    pub exit: Coord,
}

#[derive(Clone, Copy)]
pub struct SpaceshipSpec {
    pub surrounding_space_min_width: u32,
    pub size: Size,
}

impl Spaceship {
    pub fn generate<R: Rng>(spec: SpaceshipSpec, rng: &mut R) -> Self {
        let hull = generate_hull(spec.size, spec.surrounding_space_min_width, rng);
        let output_grid = add_internal_walls(&hull, rng);
        let lights = choose_lights(&output_grid, rng);
        let room_means = all_room_means(&output_grid);
        let points_of_interest = room_means.choose_multiple(rng, 2).cloned().collect::<Vec<_>>();
        let output_grid = add_connections(&output_grid, rng);
        let output_grid = add_stars(&output_grid, rng);
        Self {
            lights,
            map: output_grid,
            player_spawn: points_of_interest[0],
            exit: points_of_interest[1],
        }
    }
}
