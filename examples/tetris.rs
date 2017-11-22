extern crate prototty;
extern crate cgmath;
extern crate terminal_colour;
extern crate rand;

use std::thread;
use std::time::{Duration, Instant};
use rand::Rng;
use cgmath::Vector2;
use prototty::*;
use terminal_colour::{Colour, colours};

const BLANK_COLOUR: Colour = colours::DARK_GREY;

const PIECE_SIZE: usize = 4;

const WIDTH: u16 = 9;
const HEIGHT: u16 = 12;

const STEP_MILLIS: u64 = 500;
const ANIMATION_DELAY_MILLIS: u64 = 1000;

const ESCAPE: char = '\u{1b}';
const ETX: char = '\u{3}';

#[derive(Clone)]
struct Piece {
    colour: Colour,
    coords: [Vector2<i16>; PIECE_SIZE],
    typ: PieceType,
}

impl Piece {
    fn new(colour: Colour, coords: [(i16, i16); PIECE_SIZE], typ: PieceType) -> Self {
        let coords = [
            coords[0].into(),
            coords[1].into(),
            coords[2].into(),
            coords[3].into(),
        ];
        Self { colour, coords, typ }
    }

    fn translate(&self, offset: Vector2<i16>) -> Self {
        Self {
            colour: self.colour,
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
            Square => return self.clone(),
            _ => self.coords[1],
        };

        Self {
            colour: self.colour,
            coords: [
                Self::rotate_about(self.coords[0], offset),
                Self::rotate_about(self.coords[1], offset),
                Self::rotate_about(self.coords[2], offset),
                Self::rotate_about(self.coords[3], offset),
            ],
            typ: self.typ,
        }
    }

    fn rotate_about(coord: Vector2<i16>, offset: Vector2<i16>) -> Vector2<i16> {
        let relative = coord - offset;
        let relative = Vector2 {
            x: relative.y,
            y: 0 - relative.x,
        };
        relative + offset
    }
}

#[derive(Clone, Copy)]
enum PieceType {
    L,
    ReverseL,
    S,
    Z,
    T,
    Square,
    Line,
}

impl PieceType {
    fn piece(self) -> Piece {
        use self::PieceType::*;
        match self {
            L => Piece::new(colours::RED, [(0, 0), (0, 1), (0, 2), (1, 2)], self),
            ReverseL => Piece::new(colours::GREEN, [(1, 0), (1, 1), (1, 2), (0, 2)], self),
            S => Piece::new(colours::BLUE, [(2, 0), (1, 0), (1, 1), (0, 1)], self),
            Z => Piece::new(colours::YELLOW, [(0, 0), (1, 0), (1, 1), (2, 1)], self),
            T => Piece::new(colours::MAGENTA, [(1, 0), (0, 1), (1, 1), (2, 1)], self),
            Square => Piece::new(colours::CYAN, [(0, 0), (0, 1), (1, 0), (1, 1)], self),
            Line => Piece::new(colours::BRIGHT_BLUE, [(0, 0), (0, 1), (0, 2), (0, 3)], self),
        }
    }
}

const PIECE_TYPES: &[PieceType] = &[
    PieceType::L,
    PieceType::ReverseL,
    PieceType::S,
    PieceType::Z,
    PieceType::T,
    PieceType::Square,
    PieceType::Line,
];

fn random_piece_type<R: Rng>(rng: &mut R) -> PieceType {
    PIECE_TYPES[rng.gen::<usize>() % PIECE_TYPES.len()]
}

#[derive(Clone, Copy, Default)]
struct Cell {
    colour: Option<Colour>,
}

struct Row {
    cells: Vec<Cell>,
}

impl Row {
    fn new(width: u16) -> Self {
        let mut cells = Vec::with_capacity(width as usize);
        cells.resize(width as usize, Default::default());
        Self { cells }
    }
    fn is_full(&self) -> bool {
        self.cells.iter().all(|c| c.colour.is_some())
    }
    fn clear(&mut self) {
        self.cells.iter_mut().for_each(|c| *c = Default::default());
    }
}

struct Board {
    size: Vector2<i16>,
    rows: Vec<Row>,
    rows_swap: Vec<Row>,
    empty_swap: Vec<Row>,
}

impl Board {
    fn new(width: u16, height: u16) -> Self {

        let mut rows = Vec::with_capacity(height as usize);
        for _ in 0..height {
            rows.push(Row::new(width));
        }

        Self {
            size: Vector2::new(width, height).cast(),
            rows,
            rows_swap: Vec::new(),
            empty_swap: Vec::new(),
        }
    }

    fn get(&self, c: Vector2<i16>) -> Option<&Cell> {
        if c.x < 0 || c.y < 0 {
            return None;
        }
        let c: Vector2<usize> = c.cast();
        self.rows.get(c.y).and_then(|r| r.cells.get(c.x))
    }

    fn get_mut(&mut self, c: Vector2<i16>) -> Option<&mut Cell> {
        if c.x < 0 || c.y < 0 {
            return None;
        }
        let c: Vector2<usize> = c.cast();
        self.rows.get_mut(c.y).and_then(|r| r.cells.get_mut(c.x))
    }

    fn connects(&self, piece: &Piece) -> bool {
        piece.coords.iter().any(|c| {
            if c.y == 0 { return true; }
            self.get(c - Vector2::new(0, 1))
                .map(|c| c.colour.is_some())
                .unwrap_or(false)
        })
    }

    fn collides(&self, piece: &Piece) -> bool {
        piece.coords.iter().any(|c| {
            self.get(*c)
                .map(|c| c.colour.is_some())
                .unwrap_or(false)
                ||
                (c.x < 0 || c.x >= self.size.x)
        })
    }

    fn add_piece(&mut self, piece: Piece) {
        for coord in piece.coords.iter().cloned() {
            self.get_mut(coord).map(|c| c.colour = Some(piece.colour));
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
        for row in self.rows_swap.drain(..) {
            self.rows.push(row);
        }
        for row in self.empty_swap.drain(..) {
            self.rows.push(row);
        }
    }
}

enum StepResolution {
    GameOver,
    Continue,
}

struct Game {
    board: Board,
    piece: Option<Piece>,
}

impl Game {
    fn new(width: u16, height: u16) -> Self {
        Self {
            board: Board::new(width, height),
            piece: None,
        }
    }

    fn step<R: Rng>(&mut self, rng: &mut R) -> StepResolution {
        let (new_piece, resolution) = if let Some(piece) = self.piece.take() {

            if self.board.connects(&piece) {

                self.board.add_piece(piece);
                self.board.strip_full();

                (None, StepResolution::Continue)
            } else {
                let new_piece = piece.translate(Vector2::new(0, -1));
                (Some(new_piece), StepResolution::Continue)
            }
        } else {
            let mut piece = random_piece_type(rng).piece().translate((self.board.size.x / 2 - 1, 0).into());
            for coord in piece.coords.iter_mut() {
                coord.y = self.board.size.y - 1 - coord.y;
            }

            let mut game_over = false;
            while self.board.collides(&piece) {
                game_over = true;
                piece = piece.translate(Vector2::new(0, 1));
            }

            if game_over {
                (Some(piece), StepResolution::GameOver)
            } else {
                (Some(piece), StepResolution::Continue)
            }
        };

        self.piece = new_piece;
        resolution
    }

    fn try_move(&mut self, v: Vector2<i16>) {
        if let Some(piece) = self.piece.as_mut() {
            let new_piece = piece.translate(v);
            if !self.board.collides(&new_piece) {
                *piece = new_piece;
            }
        }
    }

    fn try_rotate(&mut self) {
        if let Some(piece) = self.piece.as_mut() {
            let new_piece = piece.rotate();
            if !self.board.collides(&new_piece) {
                *piece = new_piece;
            }
        }
    }

    fn drop(&mut self) {
        if let Some(mut piece) = self.piece.take() {
            while !self.board.connects(&piece) {
                piece = piece.translate(Vector2::new(0, -1));
            }
            self.board.add_piece(piece);
            self.board.strip_full();
        }
    }

    fn render(&self, buffer: &mut CanvasBuffer) {
        for (mut coord, canvas_cell) in buffer.enumerate_mut() {
            coord.y = self.board.size.y - 1 - coord.y;
            let board_cell = self.board.get(coord).unwrap();
            if let Some(colour) = board_cell.colour {
                canvas_cell.background_colour = colour;
            } else {
                canvas_cell.background_colour = BLANK_COLOUR;
            }
        }
        if let Some(piece) = self.piece.as_ref() {
            for mut coord in piece.coords.iter().cloned() {
                coord.y = self.board.size.y - 1 - coord.y;
                if let Some(buffer_cell) = buffer.get_mut(coord) {
                    buffer_cell.background_colour = piece.colour;
                }
            }
        }
    }
}

struct Frontend {
    end_text: TextHandle,
    canvas: CanvasHandle,
    buffer: CanvasBuffer,
    root: ElementHandle,
    container: AbsDivHandle,
}

impl Frontend {
    fn new(width: u16, height: u16) -> Self {
        let container = AbsDiv::new((width + 2, height + 2)).into_handle();
        let root = ElementHandle::from(container.clone());

        let canvas = Canvas::new((width, height)).into_handle();
        container.insert("canvas", canvas.clone(), (1, 1), None);
        let buffer = canvas.make_buffer();

        let end_text = Text::new("YOU DIED", (8, 1)).into_handle();

        Self { end_text, canvas, buffer, root, container }
    }
    fn swap(&mut self) {
        self.canvas.swap_buffer(&mut self.buffer).unwrap();
    }
    fn display_end_text(&self) {
        self.container.remove("canvas");
        self.container.insert("end_text", self.end_text.clone(), (1, 1), None);
    }
}

fn main() {
    let mut frontend = Frontend::new(WIDTH, HEIGHT);
    let mut context = Context::new().unwrap();
    let mut rng = rand::thread_rng();
    let mut game = Game::new(WIDTH, HEIGHT);
    game.step(&mut rng);

    let step_duration = Duration::from_millis(STEP_MILLIS);

    let mut step_start = Instant::now();
    let mut remaining_time = step_duration;

    loop {
        game.render(&mut frontend.buffer);
        frontend.swap();
        context.render(&frontend.root).unwrap();

        let input = match context.wait_input_timeout(remaining_time).unwrap() {
            None => {
                match game.step(&mut rng) {
                    StepResolution::Continue => (),
                    StepResolution::GameOver => {
                        game.render(&mut frontend.buffer);
                        frontend.swap();
                        context.render(&frontend.root).unwrap();
                        thread::sleep(Duration::from_millis(ANIMATION_DELAY_MILLIS));

                        frontend.display_end_text();
                        context.render(&frontend.root).unwrap();
                        thread::sleep(Duration::from_millis(ANIMATION_DELAY_MILLIS));

                        break;
                    }
                }
                step_start = Instant::now();
                remaining_time = step_duration;
                continue;
            }
            Some(input) => input,
        };

        let now = Instant::now();
        let time_since_step_start = now - step_start;
        if time_since_step_start >= step_duration {
            remaining_time = Duration::from_millis(0);
            continue;
        }
        remaining_time = step_duration - time_since_step_start;

        match input {
            Input::Char(ESCAPE) | Input::Char(ETX) => break,
            Input::Left => game.try_move(Vector2::new(-1, 0)),
            Input::Right => game.try_move(Vector2::new(1, 0)),
            Input::Up => game.try_rotate(),
            Input::Down => game.drop(),
            _ => (),
        }
    }
}
