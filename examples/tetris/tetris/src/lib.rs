use coord_2d::{Coord, Size};
use rand::Rng;
use std::mem;
use std::time::Duration;

const STEP_MILLIS: u64 = 500;
const PIECE_SIZE: usize = 4;
const WIDTH: u32 = 10;
const HEIGHT: u32 = 12;

#[derive(Clone, Copy)]
pub enum PieceType {
    L,
    J,
    S,
    Z,
    T,
    O,
    I,
}

#[derive(Clone)]
pub struct Piece {
    pub coords: [Coord; PIECE_SIZE],
    pub typ: PieceType,
}

impl Piece {
    fn new(coords: [(i32, i32); PIECE_SIZE], typ: PieceType) -> Self {
        let coords = [
            coords[0].into(),
            coords[1].into(),
            coords[2].into(),
            coords[3].into(),
        ];
        Self { coords, typ }
    }

    fn translate(&self, offset: Coord) -> Self {
        Self {
            coords: [
                self.coords[0] + offset,
                self.coords[1] + offset,
                self.coords[2] + offset,
                self.coords[3] + offset,
            ],
            typ: self.typ,
        }
    }

    fn rotate(&self) -> Self {
        use self::PieceType::*;
        let offset = match self.typ {
            // don't rotate squares
            O => return self.clone(),
            _ => self.coords[2],
        };

        Self {
            coords: [
                Self::rotate_about(self.coords[0], offset),
                Self::rotate_about(self.coords[1], offset),
                Self::rotate_about(self.coords[2], offset),
                Self::rotate_about(self.coords[3], offset),
            ],
            typ: self.typ,
        }
    }

    fn rotate_about(coord: Coord, offset: Coord) -> Coord {
        let relative = coord - offset;
        let relative = Coord {
            x: relative.y,
            y: 0 - relative.x,
        };
        relative + offset
    }
}

impl PieceType {
    fn piece(self) -> Piece {
        use self::PieceType::*;
        match self {
            L => Piece::new([(0, 0), (0, 1), (0, 2), (1, 2)], self),
            J => Piece::new([(1, 0), (1, 1), (1, 2), (0, 2)], self),
            S => Piece::new([(2, 0), (1, 0), (1, 1), (0, 1)], self),
            Z => Piece::new([(0, 0), (1, 0), (1, 1), (2, 1)], self),
            T => Piece::new([(1, 0), (0, 1), (1, 1), (2, 1)], self),
            O => Piece::new([(0, 0), (0, 1), (1, 0), (1, 1)], self),
            I => Piece::new([(0, 0), (0, 1), (0, 2), (0, 3)], self),
        }
    }
}

const PIECE_TYPES: &[PieceType] = &[
    PieceType::L,
    PieceType::J,
    PieceType::S,
    PieceType::Z,
    PieceType::T,
    PieceType::O,
    PieceType::I,
];

fn random_piece_type<R: Rng>(rng: &mut R) -> PieceType {
    PIECE_TYPES[rng.gen::<usize>() % PIECE_TYPES.len()]
}

fn random_piece<R: Rng>(rng: &mut R) -> Piece {
    random_piece_type(rng).piece()
}

#[derive(Clone, Copy, Default)]
pub struct Cell {
    pub typ: Option<PieceType>,
}

pub struct Row {
    pub cells: Vec<Cell>,
}

impl Row {
    fn new(width: u32) -> Self {
        let mut cells = Vec::with_capacity(width as usize);
        cells.resize(width as usize, Default::default());
        Self { cells }
    }
    fn is_full(&self) -> bool {
        self.cells.iter().all(|c| c.typ.is_some())
    }
    fn clear(&mut self) {
        self.cells.iter_mut().for_each(|c| *c = Default::default());
    }
}

pub struct Board {
    pub size: Size,
    pub rows: Vec<Row>,
    rows_swap: Vec<Row>,
    empty_swap: Vec<Row>,
}

impl Board {
    fn new(width: u32, height: u32) -> Self {
        let mut rows = Vec::with_capacity(height as usize);
        for _ in 0..height {
            rows.push(Row::new(width));
        }

        Self {
            size: Size::new(width, height),
            rows,
            rows_swap: Vec::new(),
            empty_swap: Vec::new(),
        }
    }

    pub fn get(&self, c: Coord) -> Option<&Cell> {
        if c.x < 0 || c.y < 0 {
            return None;
        }
        self.rows
            .get(c.y as usize)
            .and_then(|r| r.cells.get(c.x as usize))
    }

    fn get_mut(&mut self, c: Coord) -> Option<&mut Cell> {
        if c.x < 0 || c.y < 0 {
            return None;
        }
        self.rows
            .get_mut(c.y as usize)
            .and_then(|r| r.cells.get_mut(c.x as usize))
    }

    fn connects(&self, piece: &Piece) -> bool {
        piece.coords.iter().any(|c| {
            if c.y == self.size.y() as i32 - 1 {
                return true;
            }
            self.get(c + Coord::new(0, 1))
                .map(|c| c.typ.is_some())
                .unwrap_or(false)
        })
    }

    fn collides(&self, piece: &Piece) -> bool {
        piece.coords.iter().any(|c| {
            c.x < 0
                || c.x >= self.size.x() as i32
                || c.y >= self.size.y() as i32
                || self.get(*c).map(|c| c.typ.is_some()).unwrap_or(false)
        })
    }

    fn add_piece(&mut self, piece: Piece) {
        for coord in piece.coords.iter().cloned() {
            if let Some(cell) = self.get_mut(coord) {
                cell.typ = Some(piece.typ);
            }
        }
    }

    fn strip_full(&mut self) {
        for mut row in self.rows.drain(..) {
            if row.is_full() {
                row.clear();
                self.empty_swap.push(row);
            } else {
                self.rows_swap.push(row);
            }
        }
        for row in self.empty_swap.drain(..) {
            self.rows.push(row);
        }
        for row in self.rows_swap.drain(..) {
            self.rows.push(row);
        }
    }

    fn move_to_top(&self, piece: Piece) -> Piece {
        piece.translate(Coord::new(self.size.x() as i32 / 2 - 1, 0))
    }
}

enum StepResolution {
    GameOver,
    Continue,
}

pub struct GameState {
    pub board: Board,
    pub piece: Piece,
    pub next_piece: Piece,
}

impl GameState {
    fn new<R: Rng>(width: u32, height: u32, rng: &mut R) -> Self {
        let board = Board::new(width, height);
        Self {
            piece: board.move_to_top(random_piece(rng)),
            next_piece: random_piece(rng),
            board,
        }
    }

    fn step<R: Rng>(&mut self, rng: &mut R) -> StepResolution {
        if self.board.connects(&self.piece) {
            self.store_piece(rng);
            self.board.strip_full();

            let mut game_over = false;
            while self.board.collides(&self.piece) {
                game_over = true;
                self.piece = self.piece.translate(Coord::new(0, -1));
            }

            if game_over {
                return StepResolution::GameOver;
            }
        } else {
            self.piece = self.piece.translate(Coord::new(0, 1));
        }

        StepResolution::Continue
    }

    fn try_move(&mut self, v: Coord) {
        let new_piece = self.piece.translate(v);
        if !self.board.collides(&new_piece) {
            self.piece = new_piece;
        }
    }

    fn try_rotate(&mut self) {
        let new_piece = self.piece.rotate();
        if !self.board.collides(&new_piece) {
            self.piece = new_piece;
        }
    }

    fn store_piece<R: Rng>(&mut self, rng: &mut R) {
        let next_piece = mem::replace(&mut self.next_piece, random_piece(rng));
        let piece = mem::replace(&mut self.piece, self.board.move_to_top(next_piece));
        self.board.add_piece(piece);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Input {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub enum Meta {
    GameOver,
}

pub struct Step {
    remaining: Duration,
}

impl Step {
    fn new() -> Self {
        Self {
            remaining: Duration::from_millis(STEP_MILLIS),
        }
    }
    fn reduce(&mut self, period: Duration) -> bool {
        if let Some(remaining) = self.remaining.checked_sub(period) {
            self.remaining = remaining;
            false
        } else {
            self.remaining = Duration::from_millis(STEP_MILLIS);
            true
        }
    }
}

pub struct Tetris {
    step: Step,
    pub game_state: GameState,
}

impl Tetris {
    pub fn new<R: Rng>(rng: &mut R) -> Self {
        Self {
            step: Step::new(),
            game_state: GameState::new(WIDTH, HEIGHT, rng),
        }
    }

    pub fn input(&mut self, input: Input) {
        match input {
            Input::Left => self.game_state.try_move(Coord::new(-1, 0)),
            Input::Right => self.game_state.try_move(Coord::new(1, 0)),
            Input::Up => self.game_state.try_rotate(),
            Input::Down => self.game_state.try_move(Coord::new(0, 1)),
        }
    }

    pub fn tick<R: Rng>(&mut self, period: Duration, rng: &mut R) -> Option<Meta> {
        if self.step.reduce(period) {
            match self.game_state.step(rng) {
                StepResolution::Continue => (),
                StepResolution::GameOver => return Some(Meta::GameOver),
            }
        }
        None
    }
}
