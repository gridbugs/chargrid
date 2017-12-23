extern crate prototty;
extern crate tetris;
extern crate ansi_colour;
extern crate rand;

use std::collections::VecDeque;
use std::time::Duration;
use rand::Rng;
use prototty::traits::*;
use prototty::menu::*;
use prototty::text::*;
use prototty::elements::*;
use prototty::decorators::*;
use prototty::input::*;
use prototty::input::Input as ProtottyInput;

use ansi_colour::colours;
use tetris::{Tetris, PieceType, Input as TetrisInput, Meta};

const BLANK_COLOUR: Colour = colours::DARK_GREY;
const FOREGROUND_COLOUR: Colour = colours::DARK_GREY;
const BLOCK_CHAR: char = '▯';
const BLANK_CHAR: char = ' ';

const NEXT_PIECE_SIZE: Size = Size { x: 6, y: 4 };
const DEATH_ANIMATION_MILLIS: u64 = 500;
const INPUT_BUFFER_SIZE: usize = 8;

struct TetrisBoardView;
struct TetrisNextPieceView;

fn piece_colour(typ: PieceType) -> Colour {
    use tetris::PieceType::*;
    match typ {
        L => colours::RED,
        ReverseL => colours::GREEN,
        S => colours::BLUE,
        Z => colours::YELLOW,
        T => colours::MAGENTA,
        Square => colours::CYAN,
        Line => colours::BRIGHT_BLUE,
    }
}
impl View<Tetris> for TetrisBoardView {
    fn view<G: ViewGrid>(&self, tetris: &Tetris, offset: Coord, depth: i16, grid: &mut G) {
        for (i, row) in tetris.game_state.board.rows.iter().enumerate() {
            for (j, cell) in row.cells.iter().enumerate() {
                if let Some(output_cell) = grid.get_mut(offset + Coord::new(j as i16, i as i16)) {
                    if let Some(typ) = cell.typ {
                        output_cell.update_with_style(BLOCK_CHAR, depth, FOREGROUND_COLOUR, piece_colour(typ), true, false);
                    } else {
                        output_cell.update_with_colour(BLANK_CHAR, depth, FOREGROUND_COLOUR, BLANK_COLOUR);
                    }
                }
            }
        }

        for coord in tetris.game_state.piece.coords.iter().cloned() {
            if let Some(output_cell) = grid.get_mut(offset + coord) {
                output_cell.update_with_style(BLOCK_CHAR, depth, FOREGROUND_COLOUR, piece_colour(tetris.game_state.piece.typ), true, false);
            }
        }
    }
}

impl ViewSize<Tetris> for TetrisBoardView {
    fn size(&self, tetris: &Tetris) -> Size { tetris.size() }
}

impl View<Tetris> for TetrisNextPieceView {
    fn view<G: ViewGrid>(&self, tetris: &Tetris, offset: Coord, depth: i16, grid: &mut G) {
        for coord in tetris.game_state.next_piece.coords.iter().cloned() {
            if let Some(output_cell) = grid.get_mut(offset + coord) {
                output_cell.update_with_style(BLOCK_CHAR, depth, FOREGROUND_COLOUR, piece_colour(tetris.game_state.next_piece.typ), true, false);
            }
        }
    }
}

impl ViewSize<Tetris> for TetrisNextPieceView {
    fn size(&self, _: &Tetris) -> Size { NEXT_PIECE_SIZE }
}

struct Borders {
    common: Border,
    next_piece: Border,
}

impl Borders {
    fn new() -> Self {
        let mut next_piece = Border::new();
        next_piece.title = Some("next".to_string());
        Self {
            common: Border::new(),
            next_piece,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum MainMenuChoice {
    Play,
    Quit,
}

enum AppState {
    Menu,
    Game,
    GameOver,
    EndText,

}
struct Timeout {
    pub remaining: Duration,
}

impl Timeout {
    pub fn from_millis(millis: u64) -> Self {
        Self::new(Duration::from_millis(millis))
    }
    pub fn zero() -> Self {
        Self::from_millis(0)
    }
    pub fn new(remaining: Duration) -> Self {
        Self { remaining }
    }
    pub fn reduce(&mut self, duration: Duration) -> bool {
        if let Some(remaining) = self.remaining.checked_sub(duration) {
            self.remaining = remaining;
            return false;
        } else {
            self.remaining = Duration::from_millis(0);
            return true;
        }
    }
}

pub enum ControlFlow {
    Exit,
}

pub struct App {
    borders: Borders,
    main_menu: MenuInstance<MainMenuChoice>,
    state: AppState,
    timeout: Timeout,
    tetris: Tetris,
    end_text: RichText,
    input_buffer: VecDeque<TetrisInput>,
}

impl App {
    pub fn new<R: Rng>(rng: &mut R) -> Self {
        let main_menu = MenuInstance::new(Menu::smallest(
            vec![
            ("Play", MainMenuChoice::Play),
            ("Quit", MainMenuChoice::Quit),
        ])).unwrap();

        let end_text_info = TextInfo::default().bold().foreground_colour(colours::RED);
        let end_text = RichText::one_line(vec![("YOU DIED", end_text_info)]);

        Self {
            borders: Borders::new(),
            main_menu,
            state: AppState::Menu,
            timeout: Timeout::zero(),
            tetris: Tetris::new(rng),
            end_text,
            input_buffer: VecDeque::with_capacity(INPUT_BUFFER_SIZE),
        }
    }

    pub fn tick<I, R>(&mut self, inputs: I, period: Duration, rng: &mut R) -> Option<ControlFlow>
        where I: Iterator<Item=ProtottyInput>,
              R: Rng,
    {
        match self.state {
            AppState::Menu => {
                if let Some(menu_output) = self.main_menu.tick(inputs) {
                    match menu_output {
                        MenuOutput::Quit => return Some(ControlFlow::Exit),
                        MenuOutput::Cancel => (),
                        MenuOutput::Finalise(selection) => match selection {
                            MainMenuChoice::Quit => return Some(ControlFlow::Exit),
                            MainMenuChoice::Play => {
                                self.state = AppState::Game;
                            }
                        },
                    }
                }
            }
            AppState::Game => {
                for input in inputs {
                    match input {
                        ETX => return Some(ControlFlow::Exit),
                        ESCAPE => {
                            self.state = AppState::Menu;
                        }
                        ProtottyInput::Up => self.input_buffer.push_back(TetrisInput::Up),
                        ProtottyInput::Down => self.input_buffer.push_back(TetrisInput::Down),
                        ProtottyInput::Left => self.input_buffer.push_back(TetrisInput::Left),
                        ProtottyInput::Right => self.input_buffer.push_back(TetrisInput::Right),
                        _ => (),
                    }
                }
                if let Some(meta) = self.tetris.tick(self.input_buffer.drain(..), period, rng) {
                    match meta {
                        Meta::GameOver => {
                            self.timeout = Timeout::from_millis(DEATH_ANIMATION_MILLIS);
                            self.state = AppState::GameOver;
                        }
                    }
                }
            }
            AppState::GameOver => {
                if self.timeout.reduce(period) {
                    self.timeout = Timeout::from_millis(DEATH_ANIMATION_MILLIS);
                    self.state = AppState::EndText;
                }
            }
            AppState::EndText => {
                if self.timeout.reduce(period) {
                    self.tetris = Tetris::new(rng);
                    self.state = AppState::Menu;
                }
            }
        }
        None
    }
}

pub struct AppView;
impl View<App> for AppView {
    fn view<G: ViewGrid>(&self, app: &App, offset: Coord, depth: i16, grid: &mut G) {
        match app.state {
            AppState::Game | AppState::GameOver => {
                let board_renderer = Decorated::new(&TetrisBoardView, &app.borders.common);
                let next_piece_offset_x = board_renderer.size(&app.tetris).x as i16;
                board_renderer.view(&app.tetris, offset, depth, grid);
                Decorated::new(&TetrisNextPieceView, &app.borders.next_piece)
                    .view(&app.tetris, Coord { x: next_piece_offset_x, ..offset }, depth, grid);
            }
            AppState::Menu => {
                Decorated::new(&DefaultMenuInstanceView, &app.borders.common)
                    .view(&app.main_menu, offset, depth, grid);
            }
            AppState::EndText => {
                DefaultRichTextView.view(&app.end_text, offset, depth, grid);
            }
        }
    }
}
