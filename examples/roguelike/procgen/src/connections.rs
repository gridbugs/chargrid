use crate::hull::HullCell;
use direction::{CardinalDirection, Direction};
use grid_2d::{
    coord_2d::{static_axis, Axis, StaticAxis},
    Coord, Grid,
};
use rand::{seq::SliceRandom, Rng};
use std::collections::{HashMap, HashSet, VecDeque};

type RoomId = usize;
type DoorCandidateId = usize;

pub enum ConnectedCell {
    Floor,
    Wall,
    Space,
    Door,
    Window,
}

#[derive(Clone, Copy)]
enum ClassifiedCell {
    Space,
    Wall,
    Room(RoomId),
}

fn classify_cells(grid: &Grid<HullCell>) -> Grid<ClassifiedCell> {
    let mut intermediate: Grid<Option<ClassifiedCell>> = Grid::new_fn(grid.size(), |_| None);
    let mut room_count = 0;
    let mut flood_fill_buffer = VecDeque::new();
    for coord in grid.coord_iter() {
        if intermediate.get_checked(coord).is_some() {
            continue;
        }
        let classified_cell = match grid.get_checked(coord) {
            HullCell::Wall => ClassifiedCell::Wall,
            HullCell::Space => ClassifiedCell::Space,
            HullCell::Floor => {
                let classified_cell = ClassifiedCell::Room(room_count);
                flood_fill_buffer.push_back(coord);
                while let Some(coord) = flood_fill_buffer.pop_front() {
                    for direction in Direction::all() {
                        let neighbour_coord = coord + direction.coord();
                        if let Some(HullCell::Floor) = grid.get(neighbour_coord) {
                            let cell = intermediate.get_checked_mut(neighbour_coord);
                            if cell.is_none() {
                                *cell = Some(classified_cell);
                                flood_fill_buffer.push_back(neighbour_coord);
                            }
                        }
                    }
                }
                room_count += 1;
                classified_cell
            }
        };
        *intermediate.get_checked_mut(coord) = Some(classified_cell);
    }
    Grid::new_grid_map(intermediate, |maybe_cell| maybe_cell.unwrap())
}

#[derive(Debug)]
struct WindowCandidate {
    top_left: Coord,
    length: u32,
    axis: Axis,
    room: RoomId,
}

impl WindowCandidate {
    fn choose<'a, R: Rng>(&'a self, rng: &mut R) -> impl 'a + Iterator<Item = Coord> {
        let min_length = 1 + self.length / 8;
        let max_length = 1 + self.length / 3;
        let length = rng.gen_range(min_length, max_length + 1);
        let remaining_candidate_length = self.length - length;
        let min_offset = remaining_candidate_length / 4;
        let max_offset = remaining_candidate_length - min_offset;
        let offset = rng.gen_range(min_offset, max_offset + 1);
        (0..(length as i32)).map(move |i| self.top_left + Coord::new_axis(i + offset as i32, 0, self.axis.other()))
    }
}

#[derive(Debug, Clone)]
struct DoorCandidate {
    top_left: Coord,
    length: u32,
    axis: Axis,
    left_room: RoomId,
    right_room: RoomId,
}

impl DoorCandidate {
    fn choose<R: Rng>(&self, rng: &mut R) -> Coord {
        let offset = rng.gen_range(0, self.length as i32);
        self.top_left + Coord::new_axis(offset, 0, self.axis.other())
    }
    fn all<'a>(&'a self) -> impl 'a + Iterator<Item = Coord> {
        (0..(self.length as i32)).map(move |i| self.top_left + Coord::new_axis(i, 0, self.axis.other()))
    }
}

#[derive(Debug)]
enum Candidate {
    Window(WindowCandidate),
    Door(DoorCandidate),
}

impl Candidate {
    fn length_mut(&mut self) -> &mut u32 {
        match self {
            Self::Door(DoorCandidate { ref mut length, .. }) => length,
            Self::Window(WindowCandidate { ref mut length, .. }) => length,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AbstractCandidate {
    Window { room: RoomId },
    Door { left_room: RoomId, right_room: RoomId },
}

fn classify_connection_candidates_in_axis<A: StaticAxis>(grid: &Grid<ClassifiedCell>, candidates: &mut Vec<Candidate>) {
    // variables are named as if the axis A is X
    for i in 1..(grid.size().get_static::<A>().saturating_sub(1)) {
        let mut current_abstract_candidate = None;
        for j in 0..grid.size().get_static::<A::Other>() {
            let mid_coord = Coord::new_static_axis::<A>(i as i32, j as i32);
            if let ClassifiedCell::Wall = grid.get_checked(mid_coord) {
                let left_coord = mid_coord - Coord::new_static_axis::<A>(1, 0);
                let right_coord = mid_coord + Coord::new_static_axis::<A>(1, 0);
                let left = grid.get_checked(left_coord);
                let right = grid.get_checked(right_coord);
                if let Some(abstract_candidate) = match (*left, *right) {
                    (ClassifiedCell::Space, ClassifiedCell::Room(room))
                    | (ClassifiedCell::Room(room), ClassifiedCell::Space) => Some(AbstractCandidate::Window { room }),
                    (ClassifiedCell::Room(left_room), ClassifiedCell::Room(right_room)) => {
                        Some(AbstractCandidate::Door { left_room, right_room })
                    }
                    _ => None,
                } {
                    if current_abstract_candidate != Some(abstract_candidate) {
                        let new_candidate = match abstract_candidate {
                            AbstractCandidate::Window { room } => Candidate::Window(WindowCandidate {
                                top_left: mid_coord,
                                length: 0,
                                axis: A::axis(),
                                room,
                            }),
                            AbstractCandidate::Door { left_room, right_room } => Candidate::Door(DoorCandidate {
                                top_left: mid_coord,
                                length: 0,
                                axis: A::axis(),
                                left_room,
                                right_room,
                            }),
                        };
                        candidates.push(new_candidate);
                    }
                    current_abstract_candidate = Some(abstract_candidate);
                    *candidates.last_mut().unwrap().length_mut() += 1;
                    continue;
                }
            }
            current_abstract_candidate = None;
        }
    }
}

#[derive(Default)]
struct Candidates {
    door: Vec<DoorCandidate>,
    window: Vec<WindowCandidate>,
}

fn classify_connection_candidates(grid: &Grid<ClassifiedCell>) -> Candidates {
    let mut all_candidatces = Vec::new();
    classify_connection_candidates_in_axis::<static_axis::X>(grid, &mut all_candidatces);
    classify_connection_candidates_in_axis::<static_axis::Y>(grid, &mut all_candidatces);
    let mut candidates = Candidates::default();
    for candidate in all_candidatces {
        match candidate {
            Candidate::Door(door) => candidates.door.push(door),
            Candidate::Window(window) => candidates.window.push(window),
        }
    }
    candidates
}

#[derive(Debug)]
struct RoomEdge {
    to_room: RoomId,
    via: DoorCandidateId,
}

#[derive(Default, Debug)]
struct RoomNode {
    edges: Vec<RoomEdge>,
}

type DoorCandidateGraph = HashMap<RoomId, RoomNode>;

fn make_door_candidate_graph(door_candidates: &[DoorCandidate]) -> DoorCandidateGraph {
    let mut graph: DoorCandidateGraph = HashMap::new();
    for (door_candidate_id, door_candidate) in door_candidates.into_iter().enumerate() {
        graph.entry(door_candidate.left_room).or_default().edges.push(RoomEdge {
            to_room: door_candidate.right_room,
            via: door_candidate_id,
        });
        graph
            .entry(door_candidate.right_room)
            .or_default()
            .edges
            .push(RoomEdge {
                to_room: door_candidate.left_room,
                via: door_candidate_id,
            });
    }
    graph
}

fn make_random_door_candidate_graph_minimum_spanning_tree<R: Rng>(
    door_candidate_graph: &DoorCandidateGraph,
    door_candidates: &[DoorCandidate],
    rng: &mut R,
) -> HashSet<DoorCandidateId> {
    let mut mst = HashSet::new();
    let mut visited_room_ids = HashSet::new();
    let mut to_visit = vec![rng.gen_range(0, door_candidates.len())];
    while !to_visit.is_empty() {
        let door_candidate_id = to_visit.swap_remove(rng.gen_range(0, to_visit.len()));
        let door_candidate = &door_candidates[door_candidate_id];
        let new_left = visited_room_ids.insert(door_candidate.left_room);
        let new_right = visited_room_ids.insert(door_candidate.right_room);
        if !(new_left || new_right) {
            continue;
        }
        mst.insert(door_candidate_id);
        for edge in door_candidate_graph[&door_candidate.left_room]
            .edges
            .iter()
            .chain(door_candidate_graph[&door_candidate.right_room].edges.iter())
        {
            if !visited_room_ids.contains(&edge.to_room) {
                to_visit.push(edge.via);
            }
        }
    }
    mst
}

fn choose_door_candidates<R: Rng>(
    door_candidate_graph: &DoorCandidateGraph,
    door_candidates: &[DoorCandidate],
    rng: &mut R,
) -> Vec<DoorCandidateId> {
    let mut chosen_door_candidates =
        make_random_door_candidate_graph_minimum_spanning_tree(&door_candidate_graph, door_candidates, rng);
    let mut extrta_door_candidates = (0..door_candidates.len())
        .filter(|id| !chosen_door_candidates.contains(id))
        .collect::<Vec<_>>();
    extrta_door_candidates.shuffle(rng);
    let num_extra_door_candidates_to_choose = extrta_door_candidates.len() / 2;
    chosen_door_candidates.extend(extrta_door_candidates.iter().take(num_extra_door_candidates_to_choose));
    let mut chosen_door_candidates = chosen_door_candidates.into_iter().collect::<Vec<_>>();
    // the order of values from the hashset is non-deterministic, so sort first
    chosen_door_candidates.sort();
    // then deterministically shuffle
    chosen_door_candidates.shuffle(rng);
    chosen_door_candidates
}

fn trim_non_dividing_walls(grid: &Grid<HullCell>) -> Grid<HullCell> {
    let mut grid = grid.clone();
    loop {
        let mut to_clear = Vec::new();
        for (coord, cell) in grid.enumerate() {
            if let HullCell::Wall = cell {
                let mut wall_neighbour_count = 0;
                for direction in CardinalDirection::all() {
                    let neighbour_coord = coord + direction.coord();
                    if let Some(HullCell::Wall) = grid.get(neighbour_coord) {
                        wall_neighbour_count += 1;
                    }
                }
                if wall_neighbour_count <= 1 {
                    to_clear.push(coord);
                }
            }
        }
        if to_clear.is_empty() {
            break;
        }
        for coord in to_clear {
            *grid.get_checked_mut(coord) = HullCell::Floor;
        }
    }
    grid
}

fn place_door<R: Rng>(candidate: &DoorCandidate, grid: &mut Grid<ConnectedCell>, rng: &mut R) {
    let coord = candidate.choose(rng);
    *grid.get_checked_mut(coord) = ConnectedCell::Door;
}

fn place_window<R: Rng>(candidate: &WindowCandidate, grid: &mut Grid<ConnectedCell>, rng: &mut R) {
    if rng.gen_range(0, 3) > 0 {
        for coord in candidate.choose(rng) {
            *grid.get_checked_mut(coord) = ConnectedCell::Window;
        }
    }
}

pub fn add_connections<R: Rng>(grid: &Grid<HullCell>, rng: &mut R) -> Grid<ConnectedCell> {
    let classified = classify_cells(grid);
    let candidates = classify_connection_candidates(&classified);
    let door_candidate_graph = make_door_candidate_graph(&candidates.door);
    let chosen_door_candidates = choose_door_candidates(&door_candidate_graph, &candidates.door, rng);
    let mut grid = grid.clone();
    for &door_candidate_id in chosen_door_candidates.iter() {
        if rng.gen_range(0, 10) == 0 {
            let candidate = &candidates.door[door_candidate_id];
            for coord in candidate.all() {
                *grid.get_checked_mut(coord) = HullCell::Floor;
            }
        }
    }
    let grid = trim_non_dividing_walls(&grid);
    let classified = classify_cells(&grid);
    let candidates = classify_connection_candidates(&classified);
    let door_candidate_graph = make_door_candidate_graph(&candidates.door);
    let chosen_door_candidates = choose_door_candidates(&door_candidate_graph, &candidates.door, rng);
    let mut grid = Grid::new_grid_map_ref(&grid, |cell| match cell {
        HullCell::Floor => ConnectedCell::Floor,
        HullCell::Wall => ConnectedCell::Wall,
        HullCell::Space => ConnectedCell::Space,
    });
    for &door_candidate_id in chosen_door_candidates.iter() {
        place_door(&candidates.door[door_candidate_id], &mut grid, rng);
    }
    for window_candidate in candidates.window.iter() {
        place_window(window_candidate, &mut grid, rng);
    }
    grid
}
