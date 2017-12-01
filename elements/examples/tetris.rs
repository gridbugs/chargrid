extern crate prototty;
extern crate prototty_elements;
extern crate cgmath;
extern crate ansi_colour;
extern crate rand;

use std::mem;
use std::thread;
use std::time::{Duration, Instant};
use rand::Rng;
use cgmath::Vector2;
use prototty::*;
use prototty_elements::*;
use ansi_colour::{Colour, colours};

const BLANK_COLOUR: Colour = colours::DARK_GREY;
const FOREGROUND_COLOUR: Colour = colours::DARK_GREY;
const BORDER_COLOUR: Colour = colours::WHITE;
const BORDER_BACKGROUND: Colour = colours::BLACK;
const BLOCK_CHAR: char = '▯';

const PIECE_SIZE: usize = 4;

const WIDTH: u16 = 10;
const HEIGHT: u16 = 12;

const STEP_MILLIS: u64 = 500;

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
            _ => self.coords[2],
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

fn random_piece<R: Rng>(rng: &mut R) -> Piece {
    random_piece_type(rng).piece()
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
            if c.y == self.size.y - 1 {
                return true;
            }
            self.get(c + Vector2::new(0, 1))
                .map(|c| c.colour.is_some())
                .unwrap_or(false)
        })
    }

    fn collides(&self, piece: &Piece) -> bool {
        piece.coords.iter().any(|c| {
            c.x < 0 || c.x >= self.size.x ||
            c.y >= self.size.y ||
            self.get(*c)
                .map(|c| c.colour.is_some())
                .unwrap_or(false)
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
        for row in self.empty_swap.drain(..) {
            self.rows.push(row);
        }
        for row in self.rows_swap.drain(..) {
            self.rows.push(row);
        }
    }

    fn move_to_top(&self, piece: Piece) -> Piece {
        piece.translate(Vector2::new(self.size.x as i16 / 2 - 1, 0))
    }
}

enum StepResolution {
    GameOver,
    Continue,
}

struct Game {
    board: Board,
    piece: Piece,
    next_piece: Piece,
}

impl Game {
    fn new<R: Rng>(width: u16, height: u16, rng: &mut R) -> Self {
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
                self.piece = self.piece.translate(Vector2::new(0, -1));
            }

            if game_over {
                return StepResolution::GameOver;
            }

        } else {
            self.piece = self.piece.translate(Vector2::new(0, 1));
        }

        StepResolution::Continue
    }

    fn try_move(&mut self, v: Vector2<i16>) {
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

    fn render(&self, buffer: &mut CanvasBuffer) {
        for (coord, canvas_cell) in buffer.enumerate_mut() {
            let board_cell = self.board.get(coord).unwrap();
            if let Some(colour) = board_cell.colour {
                canvas_cell.background_colour = colour;
                canvas_cell.character = BLOCK_CHAR;
                canvas_cell.foreground_colour = FOREGROUND_COLOUR;
            } else {
                canvas_cell.character = ' ';
                canvas_cell.background_colour = BLANK_COLOUR;
            }
        }
        for coord in self.piece.coords.iter().cloned() {
            if let Some(buffer_cell) = buffer.get_mut(coord) {
                buffer_cell.background_colour = self.piece.colour;
                buffer_cell.foreground_colour = FOREGROUND_COLOUR;
                buffer_cell.character = BLOCK_CHAR;
            }
        }
    }

    fn render_next(&self, buffer: &mut CanvasBuffer) {
        for cell in buffer.iter_mut() {
            cell.character = ' ';
            cell.foreground_colour = BLANK_COLOUR;
            cell.background_colour = BLANK_COLOUR;
        }
        for coord in self.next_piece.coords.iter().cloned() {
            if let Some(cell) = buffer.get_mut(coord + Vector2::new(1, 0)) {
                cell.character = BLOCK_CHAR;
                cell.foreground_colour = FOREGROUND_COLOUR;
                cell.background_colour = self.next_piece.colour;
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum MainMenuChoice {
    Play,
    Quit,
}

struct Model {
    game_canvas: Border<Canvas>,
    piece_canvas: Border<Canvas>,
}

impl View for Model {
    fn view<G: ViewGrid>(&self, offset: Vector2<i16>, depth: i16, grid: &mut G) {
        self.game_canvas.view(offset + Vector2::new(1, 1), depth, grid);
        let piece_offset = Vector2::new(self.game_canvas.size().x + 1, 1).cast();
        self.piece_canvas.view(offset + piece_offset, depth, grid);
    }
    fn size(&self) -> Vector2<u16> {
        Vector2 {
            x: self.game_canvas.size().x + self.piece_canvas.size().x + 2,
            y: self.game_canvas.size().y + self.piece_canvas.size().y + 2,
        }
    }
}

struct Frontend {
    context: Context,
    model: Model,
    game_buffer: CanvasBuffer,
    piece_buffer: CanvasBuffer,
}

impl Frontend {
    fn new(width: u16, height: u16) -> Self {
        let context = Context::new().unwrap();
        let mut model = Model {
            game_canvas: Border::new(Canvas::new((width, height))),
            piece_canvas: Border::new(Canvas::new((6, 4))),
        };

        model.piece_canvas.title = Some("next".to_string());
        model.piece_canvas.foreground_colour = BORDER_COLOUR;
        model.piece_canvas.background_colour = BORDER_BACKGROUND;
        model.game_canvas.foreground_colour = BORDER_COLOUR;
        model.game_canvas.background_colour = BORDER_BACKGROUND;

        let game_buffer = model.game_canvas.child.make_buffer();
        let piece_buffer = model.piece_canvas.child.make_buffer();
        Self {
            context,
            model,
            game_buffer,
            piece_buffer,
        }
    }
    fn display_end_text(&mut self) {
        self.context.render(&RichText::one_line(
                vec![("YOU DIED", TextInfo::default().bold().foreground_colour(colours::RED))])).unwrap();
    }
    fn render(&mut self, game: &Game) {
        game.render(&mut self.game_buffer);
        self.model.game_canvas.child.swap_buffer(&mut self.game_buffer).unwrap();
        game.render_next(&mut self.piece_buffer);
        self.model.piece_canvas.child.swap_buffer(&mut self.piece_buffer).unwrap();
        self.context.render(&self.model).unwrap();
    }
    fn main_menu(&mut self) -> MainMenuChoice {
        let mut menu = Border::new(MenuInstance::new(Menu::smallest(vec![
            ("Play", MainMenuChoice::Play),
            ("Quit", MainMenuChoice::Quit),
        ])));

        match self.context.run_menu(&mut menu, |b| &mut b.child).unwrap() {
            MenuAction::Select(x) => x,
            _ => MainMenuChoice::Quit,
        }
    }
    fn wait_input_timeout(&mut self, duration: Duration) -> Option<Input> {
        self.context.wait_input_timeout(duration).unwrap()
    }
}

fn main() {
    let mut frontend = Frontend::new(WIDTH, HEIGHT);
    let mut rng = rand::thread_rng();
    loop {
        match frontend.main_menu() {
            MainMenuChoice::Quit => break,
            MainMenuChoice::Play => (),
        }

        let mut game = Game::new(WIDTH, HEIGHT, &mut rng);

        let step_duration = Duration::from_millis(STEP_MILLIS);

        let mut step_start = Instant::now();
        let mut remaining_time = step_duration;

        loop {
            frontend.render(&game);

            let input = match frontend.wait_input_timeout(remaining_time) {
                None => {
                    match game.step(&mut rng) {
                        StepResolution::Continue => (),
                        StepResolution::GameOver => {
                            frontend.render(&game);
                            thread::sleep(Duration::from_millis(500));
                            frontend.display_end_text();
                            thread::sleep(Duration::from_millis(1000));

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
                Input::Down => game.try_move(Vector2::new(0, 1)),
                _ => (),
            }
        }
    }
}
