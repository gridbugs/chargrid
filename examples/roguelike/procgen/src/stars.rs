use crate::connections::ConnectedCell;
use grid_2d::Grid;
use rand::Rng;

pub enum StarCell {
    Wall,
    Floor,
    Space,
    Door,
    Window,
    Star,
}

pub fn add_stars<R: Rng>(grid: &Grid<ConnectedCell>, rng: &mut R) -> Grid<StarCell> {
    Grid::new_grid_map_ref(grid, |cell| match cell {
        ConnectedCell::Door => StarCell::Door,
        ConnectedCell::Wall => StarCell::Wall,
        ConnectedCell::Floor => StarCell::Floor,
        ConnectedCell::Window => StarCell::Window,
        ConnectedCell::Space => {
            if rng.gen_range(0, 20) == 0 {
                StarCell::Star
            } else {
                StarCell::Space
            }
        }
    })
}
