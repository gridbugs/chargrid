use direction::Direction;
use grid_2d::{Coord, Grid, Size};
use rand::Rng;
use std::collections::HashSet;
use std::num::NonZeroU32;
use wfc::{overlapping::OverlappingPatterns, retry, wrap, ForbidNothing, RunOwn};

#[rustfmt::skip]
const INPUT: &[&str] = &[
    ".............",
    ".............",
    "..#####......",
    "..#...#......",
    "..#...#......",
    "..#...#......",
    "..#...#####..",
    "..#.......#..",
    "..##......#..",
    "...##.....#..",
    "....#######..",
    ".............",
    ".............",
];

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum GenerationCell {
    Closed,
    Open,
}

fn input_grid_from_strs(input: &[&str]) -> Grid<GenerationCell> {
    let width = input[0].len();
    let height = input.len();
    let size = Size::new(width as u32, height as u32);
    let mut grid = Grid::new_clone(size, GenerationCell::Open);
    for (y, row) in input.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            let coord = Coord::new(x as i32, y as i32);
            let cell = match ch {
                '.' => GenerationCell::Open,
                '#' => GenerationCell::Closed,
                ch => panic!("unexpected char: {}", ch),
            };
            *grid.get_checked_mut(coord) = cell;
        }
    }
    grid
}

fn wfc_map<R: Rng>(
    input_grid: Grid<GenerationCell>,
    output_size: Size,
    pattern_size: NonZeroU32,
    rng: &mut R,
) -> Grid<GenerationCell> {
    let mut output_grid = Grid::new_clone(output_size, GenerationCell::Open);
    let overlapping_patterns = OverlappingPatterns::new_all_orientations(input_grid, pattern_size);
    let global_stats = overlapping_patterns.global_stats();
    let run = RunOwn::new_wrap_forbid(output_size, &global_stats, wrap::WrapXY, ForbidNothing, rng);
    let wave = run.collapse_retrying(retry::Forever, rng);
    for (coord, wave_cell) in wave.grid().enumerate() {
        let pattern_id = wave_cell.chosen_pattern_id().expect("unexpected contradiction");
        let cell = overlapping_patterns.pattern_top_left_value(pattern_id);
        *output_grid.get_checked_mut(coord) = *cell;
    }
    output_grid
}

fn keep_largest_enclosed_area(grid: &Grid<GenerationCell>) -> Grid<GenerationCell> {
    let mut visited_ids: Grid<Option<usize>> = Grid::new_clone(grid.size(), None);
    let mut flood_fill_buffer = Vec::new();
    let mut current_id = 0usize;
    let mut counts_by_id = Vec::new();
    for (coord, cell) in grid.enumerate() {
        if let GenerationCell::Open = cell {
            if visited_ids.get_checked(coord).is_none() {
                flood_fill_buffer.push(coord);
                *visited_ids.get_checked_mut(coord) = Some(current_id);
                let mut count = 0usize;
                while let Some(coord) = flood_fill_buffer.pop() {
                    count += 1;
                    for direction in Direction::all() {
                        let next_coord = coord + direction.coord();
                        match grid.get(next_coord) {
                            None | Some(GenerationCell::Closed) => continue,
                            Some(GenerationCell::Open) => (),
                        }
                        let maybe_visited_id = visited_ids.get_checked_mut(next_coord);
                        if maybe_visited_id.is_none() {
                            *maybe_visited_id = Some(current_id);
                            flood_fill_buffer.push(next_coord);
                        }
                    }
                }
                counts_by_id.push(count);
                current_id += 1;
            }
        }
    }
    let (id_of_largest_area, _count) = counts_by_id
        .iter()
        .enumerate()
        .max_by_key(|&(_, count)| count)
        .expect("found no enclosed areas");
    let grid_keeping_largest_enclosed_area = Grid::new_grid_map_ref(&visited_ids, |&maybe_id| match maybe_id {
        Some(id) => {
            if id == id_of_largest_area {
                GenerationCell::Open
            } else {
                GenerationCell::Closed
            }
        }
        None => GenerationCell::Closed,
    });
    grid_keeping_largest_enclosed_area
}

fn grow_enclosed_areas(grid: &Grid<GenerationCell>, by: usize) -> Grid<GenerationCell> {
    let mut grid = grid.clone();
    let mut coords_to_grow = grid
        .enumerate()
        .filter_map(|(coord, cell)| {
            if let GenerationCell::Open = cell {
                Some((coord, by))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    while let Some((coord, remaining)) = coords_to_grow.pop() {
        if remaining == 0 {
            continue;
        }
        for direction in Direction::all() {
            let next_coord = coord + direction.coord();
            match grid.get_mut(next_coord) {
                None | Some(GenerationCell::Open) => continue,
                Some(cell @ GenerationCell::Closed) => {
                    *cell = GenerationCell::Open;
                    coords_to_grow.push((next_coord, remaining - 1));
                }
            }
        }
    }
    grid
}

fn wrap_in_closed_area(grid: &Grid<GenerationCell>) -> Grid<GenerationCell> {
    Grid::new_fn(grid.size() + Size::new(2, 2), |coord| {
        if let Some(cell) = grid.get(coord - Coord::new(1, 1)) {
            *cell
        } else {
            GenerationCell::Closed
        }
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HullCell {
    Wall,
    Floor,
    Space,
}

fn strip_walls_from_outside(grid: &Grid<GenerationCell>) -> Grid<HullCell> {
    Grid::new_grid_map_ref_with_coord(grid, |coord, cell| match cell {
        GenerationCell::Open => HullCell::Floor,
        GenerationCell::Closed => {
            for direction in Direction::all() {
                let neighbour_coord = coord + direction.coord();
                if let Some(GenerationCell::Open) = grid.get(neighbour_coord) {
                    return HullCell::Wall;
                }
            }
            HullCell::Space
        }
    })
}

fn remove_small_closed_areas(grid: &Grid<GenerationCell>, min_count: usize) -> Grid<GenerationCell> {
    let mut grid = grid.clone();
    let mut visited_ids: Grid<Option<usize>> = Grid::new_clone(grid.size(), None);
    let mut flood_fill_buffer = Vec::new();
    let mut current_id = 0usize;
    let mut counts_by_id = Vec::new();
    let mut ids_to_remove = HashSet::new();
    for (coord, cell) in grid.enumerate() {
        if let GenerationCell::Open = cell {
            if visited_ids.get_checked(coord).is_none() {
                flood_fill_buffer.push(coord);
                *visited_ids.get_checked_mut(coord) = Some(current_id);
                let mut count = 0usize;
                while let Some(coord) = flood_fill_buffer.pop() {
                    count += 1;
                    for direction in Direction::all() {
                        let next_coord = coord + direction.coord();
                        match grid.get(next_coord) {
                            None | Some(GenerationCell::Open) => continue,
                            Some(GenerationCell::Closed) => (),
                        }
                        let maybe_visited_id = visited_ids.get_checked_mut(next_coord);
                        if maybe_visited_id.is_none() {
                            *maybe_visited_id = Some(current_id);
                            flood_fill_buffer.push(next_coord);
                        }
                    }
                }
                counts_by_id.push(count);
                if count < min_count {
                    ids_to_remove.insert(current_id);
                }
                current_id += 1;
            }
        }
    }
    for (grid_cell, visited_id) in grid.iter_mut().zip(visited_ids.iter()) {
        if let Some(visited_id) = visited_id {
            if ids_to_remove.contains(&visited_id) {
                *grid_cell = GenerationCell::Open;
            }
        }
    }
    grid
}

fn surround_by_space(grid: &Grid<HullCell>, width: u32) -> Grid<HullCell> {
    let offset = Size::new(width, width);
    Grid::new_fn(grid.size() + offset * 2, |coord| {
        if let Some(&cell) = grid.get(coord - offset.to_coord().unwrap()) {
            cell
        } else {
            HullCell::Space
        }
    })
}

fn generate_hull_internal<R: Rng>(
    input_grid: Grid<GenerationCell>,
    output_size: Size,
    space_width: u32,
    pattern_size: NonZeroU32,
    rng: &mut R,
) -> Grid<HullCell> {
    let output_grid = wfc_map(
        input_grid,
        output_size - Size::new(space_width + 1, space_width + 1) * 2,
        pattern_size,
        rng,
    );
    let output_grid = keep_largest_enclosed_area(&output_grid);
    let output_grid = grow_enclosed_areas(&output_grid, 1);
    let output_grid = remove_small_closed_areas(&output_grid, 40);
    let output_grid = wrap_in_closed_area(&output_grid);
    let output_grid = strip_walls_from_outside(&output_grid);
    let output_grid = surround_by_space(&output_grid, space_width);
    output_grid
}

pub fn generate_hull<R: Rng>(output_size: Size, space_width: u32, rng: &mut R) -> Grid<HullCell> {
    let input_grid = input_grid_from_strs(INPUT);
    let pattern_size = NonZeroU32::new(4).unwrap();
    generate_hull_internal(input_grid, output_size, space_width, pattern_size, rng)
}
