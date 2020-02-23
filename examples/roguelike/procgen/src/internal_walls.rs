use crate::hull::HullCell;
use grid_2d::{
    coord_2d::{static_axis, Axis, StaticAxis},
    Coord, Grid, Size,
};
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::BTreeSet;
use std::marker::PhantomData;

#[derive(Debug)]
enum Side {
    High,
    Low,
}

#[derive(Debug)]
struct ExternalWall {
    top_left: Coord,
    length: usize,
    inside: Side,
}

fn classify_walls<A: StaticAxis>(grid: &Grid<HullCell>) -> ExternalWallsInAxis<A> {
    let mut walls = Vec::new();
    for i in 0..grid.size().get_static::<A::Other>() {
        let mut consecutive_count = 0;
        for j in 0..grid.size().get_static::<A>() {
            let coord = Coord::new_static_axis::<A>(j as i32, i as i32);
            let high_coord = coord + Coord::new_static_axis::<A>(0, 1);
            let low_coord = coord - Coord::new_static_axis::<A>(0, 1);
            if let Some(HullCell::Wall) = grid.get(high_coord) {
                consecutive_count = 0;
                continue;
            }
            if let Some(HullCell::Wall) = grid.get(low_coord) {
                consecutive_count = 0;
                continue;
            }
            if *grid.get_checked(coord) == HullCell::Wall {
                if consecutive_count == 0 {
                    walls.push(ExternalWall {
                        top_left: coord,
                        length: 0,
                        inside: Side::High,
                    });
                }
                consecutive_count += 1;
                let current_wall = walls.last_mut().unwrap();
                current_wall.length = consecutive_count;
                let high_direction = Coord::new_static_axis::<A>(0, 1);
                match grid.get(coord + high_direction) {
                    Some(HullCell::Space) | None => current_wall.inside = Side::Low,
                    Some(HullCell::Floor) => current_wall.inside = Side::High,
                    Some(HullCell::Wall) => (),
                }
            } else {
                consecutive_count = 0;
            }
        }
    }
    ExternalWallsInAxis {
        walls,
        axis: PhantomData,
    }
}

#[derive(Debug)]
struct Rect {
    coord: Coord,
    size: Size,
}

struct ExternalWallsInAxis<A: StaticAxis> {
    axis: PhantomData<A>,
    walls: Vec<ExternalWall>,
}

struct ExternalWalls {
    horizontal: ExternalWallsInAxis<static_axis::X>,
    vertical: ExternalWallsInAxis<static_axis::Y>,
}

struct InternalWallCandidatesInAxis<A: StaticAxis> {
    axis_aligned_with_walls: PhantomData<A>,
    candidates: Vec<u32>,
}

struct InternalWallCandidates {
    y_coord_of_horizontal_wall_candidates: InternalWallCandidatesInAxis<static_axis::X>,
    x_coord_of_vertical_wall_candidates: InternalWallCandidatesInAxis<static_axis::Y>,
}

fn wall_intersects_rect<A: StaticAxis>(wall: &ExternalWall, rect: &Rect) -> bool {
    // variables are named as if wall_axis is X
    let top = rect.coord.get_static::<A::Other>();
    let bottom = top + rect.size.get_static::<A::Other>() as i32 - 1;
    let left = rect.coord.get_static::<A>();
    let right = left + rect.size.get_static::<A>() as i32 - 1;
    let wall_y = wall.top_left.get_static::<A::Other>();
    if wall_y < top || wall_y > bottom {
        return false;
    }
    let wall_start_x = wall.top_left.get_static::<A>();
    let wall_end_x = wall_start_x + wall.length as i32 - 1;
    if wall_start_x > right || wall_end_x < left {
        return false;
    }
    true
}

fn find_internal_wall_candidates<A: StaticAxis>(
    walls: &ExternalWallsInAxis<A>,
    rect: &Rect,
    min_rect_size: Size,
) -> InternalWallCandidatesInAxis<A> {
    let min_distance_from_wall = min_rect_size.get_static::<A::Other>();
    let top_left = rect.coord.get_static::<A::Other>() as u32;
    let low = top_left + min_distance_from_wall;
    let high = (top_left + rect.size.get_static::<A::Other>()).saturating_sub(min_distance_from_wall);
    let mut candidates = (low..high).collect::<BTreeSet<_>>();
    for wall in &walls.walls {
        if !wall_intersects_rect::<A>(wall, rect) {
            continue;
        }
        let position = wall.top_left.get_static::<A::Other>() as u32;
        let range = match wall.inside {
            Side::Low => ((position - min_distance_from_wall)..=(position - 1)),
            Side::High => ((position + 1)..=(position + min_distance_from_wall)),
        };
        for index in range {
            candidates.remove(&index);
        }
    }
    InternalWallCandidatesInAxis {
        axis_aligned_with_walls: PhantomData,
        candidates: candidates.into_iter().collect::<Vec<_>>(),
    }
}

impl ExternalWalls {
    fn classify(grid: &Grid<HullCell>) -> Self {
        let horizontal = classify_walls::<static_axis::X>(grid);
        let vertical = classify_walls::<static_axis::Y>(grid);
        Self { horizontal, vertical }
    }

    fn internal_wall_candidates(&self, rect: &Rect, min_rect_size: Size) -> InternalWallCandidates {
        let y_coord_of_horizontal_wall_candidates =
            find_internal_wall_candidates(&self.horizontal, rect, min_rect_size);
        let x_coord_of_vertical_wall_candidates = find_internal_wall_candidates(&self.vertical, rect, min_rect_size);
        InternalWallCandidates {
            y_coord_of_horizontal_wall_candidates,
            x_coord_of_vertical_wall_candidates,
        }
    }
}

struct Split {
    left: Rect,
    right: Rect,
    internal_wall: InternalWall,
}

fn split_rect_with_wall<R: Rng, A: StaticAxis>(
    split_candidates: &InternalWallCandidatesInAxis<A>,
    rect: Rect,
    rng: &mut R,
) -> Split {
    let &split_position = split_candidates
        .candidates
        .choose(rng)
        .expect("split_candidates should not be empty");
    let left = Rect {
        coord: rect.coord,
        size: rect
            .size
            .set_static::<A::Other>(split_position - rect.coord.get_static::<A::Other>() as u32),
    };
    let right = Rect {
        coord: rect.coord + left.size.to_coord().unwrap().set_static::<A>(0) + Coord::new_static_axis::<A>(0, 1),
        size: rect.size - left.size.set_static::<A>(0) - Size::new_static_axis::<A>(0, 1),
    };
    let internal_wall = InternalWall {
        top_left: rect.coord.set_static::<A::Other>(split_position as i32),
        length: rect.size.get_static::<A>(),
        axis: A::axis(),
    };
    Split {
        left,
        right,
        internal_wall,
    }
}

struct InternalWall {
    top_left: Coord,
    length: u32,
    axis: Axis,
}

impl InternalWall {
    fn draw(&self, grid: &mut Grid<HullCell>) {
        let step = Coord::new_axis(1, 0, self.axis);
        for i in 0..self.length {
            let coord = self.top_left + step * i as i32;
            let cell = grid.get_checked_mut(coord);
            if let HullCell::Floor = cell {
                *cell = HullCell::Wall;
            }
        }
    }
}

fn add_internal_walls_rec<R: Rng>(
    external_walls: &ExternalWalls,
    rect: Rect,
    min_rect_size: Size,
    internal_walls: &mut Vec<InternalWall>,
    rng: &mut R,
) {
    assert!(rect.size.width() >= min_rect_size.width());
    assert!(rect.size.height() >= min_rect_size.height());
    let internal_wall_candidates = external_walls.internal_wall_candidates(&rect, min_rect_size);
    let candidates_x = &internal_wall_candidates.y_coord_of_horizontal_wall_candidates;
    let candidates_y = &internal_wall_candidates.x_coord_of_vertical_wall_candidates;
    let Split {
        left,
        right,
        internal_wall,
    } = if rect.size.width() < min_rect_size.width() * 2 + 1 {
        if rect.size.height() < min_rect_size.height() * 2 + 1 {
            return;
        } else {
            if candidates_x.candidates.is_empty() {
                return;
            } else {
                split_rect_with_wall(candidates_x, rect, rng)
            }
        }
    } else {
        if rect.size.height() < min_rect_size.height() * 2 + 1 {
            if candidates_y.candidates.is_empty() {
                return;
            } else {
                split_rect_with_wall(candidates_y, rect, rng)
            }
        } else {
            if candidates_x.candidates.is_empty() {
                if candidates_y.candidates.is_empty() {
                    return;
                } else {
                    split_rect_with_wall(candidates_y, rect, rng)
                }
            } else {
                if candidates_y.candidates.is_empty() {
                    split_rect_with_wall(candidates_x, rect, rng)
                } else {
                    if rng.gen() {
                        split_rect_with_wall(candidates_x, rect, rng)
                    } else {
                        split_rect_with_wall(candidates_y, rect, rng)
                    }
                }
            }
        }
    };
    assert!(left.size.width() >= min_rect_size.width());
    assert!(left.size.height() >= min_rect_size.height());
    assert!(right.size.width() >= min_rect_size.width());
    assert!(right.size.height() >= min_rect_size.height());
    internal_walls.push(internal_wall);
    add_internal_walls_rec(external_walls, left, min_rect_size, internal_walls, rng);
    add_internal_walls_rec(external_walls, right, min_rect_size, internal_walls, rng);
}

pub fn add_internal_walls<R: Rng>(grid: &Grid<HullCell>, rng: &mut R) -> Grid<HullCell> {
    let external_walls = ExternalWalls::classify(grid);
    let mut internal_walls = Vec::new();
    add_internal_walls_rec(
        &external_walls,
        Rect {
            coord: Coord::new(0, 0),
            size: grid.size(),
        },
        Size::new(4, 4),
        &mut internal_walls,
        rng,
    );
    let mut grid = grid.clone();
    for wall in internal_walls {
        wall.draw(&mut grid);
    }
    grid
}
