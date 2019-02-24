extern crate prototty;
extern crate rand;
extern crate tetris;

use prototty::*;
use prototty_inputs::*;
use rand::Rng;
use std::collections::VecDeque;
use std::time::Duration;
use tetris::{Input as TetrisInput, Meta, PieceType, Tetris};

const BLANK_FOREGROUND_COLOUR: Rgb24 = rgb24(24, 24, 24);
const FOREGROUND_COLOUR: Rgb24 = colours::WHITE;
const BACKGROUND_COLOUR: Rgb24 = colours::BLACK;
const BLOCK_CHAR: char = '-';
const BLANK_CHAR: char = '-';

const NEXT_PIECE_SIZE: [u32; 2] = [6, 4];
const DEATH_ANIMATION_MILLIS: u64 = 500;
const INPUT_BUFFER_SIZE: usize = 8;

struct TetrisBoardView;
struct TetrisNextPieceView;

fn piece_colour(typ: PieceType) -> Rgb24 {
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
    fn view<G: ViewGrid>(&mut self, tetris: &Tetris, offset: Coord, depth: i32, grid: &mut G) {
        for (i, row) in tetris.game_state.board.rows.iter().enumerate() {
            for (j, cell) in row.cells.iter().enumerate() {
                let mut cell_info = ViewCell::new().with_bold(true);
                if let Some(typ) = cell.typ {
                    cell_info.character = Some(BLOCK_CHAR);
                    cell_info.foreground = Some(FOREGROUND_COLOUR);
                    cell_info.background = Some(piece_colour(typ));
                } else {
                    cell_info.character = Some(BLANK_CHAR);
                    cell_info.foreground = Some(BLANK_FOREGROUND_COLOUR);
                    cell_info.background = Some(BACKGROUND_COLOUR);
                }
                grid.set_cell(offset + Coord::new(j as i32, i as i32), depth, cell_info);
            }
        }
        for coord in tetris.game_state.piece.coords.iter().cloned() {
            let cell_info = ViewCell {
                character: Some(BLOCK_CHAR),
                bold: Some(true),
                underline: Some(false),
                foreground: Some(FOREGROUND_COLOUR),
                background: Some(piece_colour(tetris.game_state.piece.typ)),
            };
            grid.set_cell(offset + coord, depth, cell_info);
        }
    }
}

impl ViewSize<Tetris> for TetrisBoardView {
    fn size(&mut self, tetris: &Tetris) -> Size {
        tetris.size().into()
    }
}

impl View<Tetris> for TetrisNextPieceView {
    fn view<G: ViewGrid>(&mut self, tetris: &Tetris, offset: Coord, depth: i32, grid: &mut G) {
        let offset = offset + Coord::new(1, 0);
        for coord in tetris.game_state.next_piece.coords.iter().cloned() {
            let cell_info = ViewCell {
                character: Some(BLOCK_CHAR),
                bold: Some(true),
                underline: Some(false),
                foreground: Some(FOREGROUND_COLOUR),
                background: Some(piece_colour(tetris.game_state.next_piece.typ)),
            };
            grid.set_cell(offset + coord, depth, cell_info);
        }
    }
}

impl ViewSize<Tetris> for TetrisNextPieceView {
    fn size(&mut self, _: &Tetris) -> Size {
        NEXT_PIECE_SIZE.into()
    }
}

struct Borders {
    common: Decorated<TetrisBoardView, Border>,
    next_piece: Decorated<TetrisNextPieceView, Border>,
    menu: Decorated<DefaultMenuInstanceView, Border>,
}

impl Borders {
    fn new() -> Self {
        let mut next_piece = Border::new();
        next_piece.title = Some("next".to_string());

        let common = Border::new();

        Self {
            common: Decorated::new(TetrisBoardView, common.clone()),
            next_piece: Decorated::new(TetrisNextPieceView, next_piece),
            menu: Decorated::new(DefaultMenuInstanceView::new(), common),
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
    main_menu: MenuInstance<MainMenuChoice>,
    state: AppState,
    timeout: Timeout,
    tetris: Tetris,
    end_text: RichText,
    input_buffer: VecDeque<TetrisInput>,
}

impl App {
    pub fn new<R: Rng>(rng: &mut R) -> Self {
        let mut main_menu = Menu::smallest(vec![
            ("Play", MainMenuChoice::Play),
            ("Quit", MainMenuChoice::Quit),
        ]);

        main_menu.selected_info = TextInfo::default()
            .foreground_colour(colours::BLACK)
            .background_colour(colours::WHITE)
            .bold()
            .underline();

        let main_menu = MenuInstance::new(main_menu).unwrap();

        let end_text_info = TextInfo::default().bold().foreground_colour(colours::RED);
        let end_text = RichText::one_line(vec![("YOU DIED", end_text_info)]);

        Self {
            main_menu,
            state: AppState::Menu,
            timeout: Timeout::zero(),
            tetris: Tetris::new(rng),
            end_text,
            input_buffer: VecDeque::with_capacity(INPUT_BUFFER_SIZE),
        }
    }

    pub fn tick<I, R>(
        &mut self,
        inputs: I,
        period: Duration,
        view: &AppView,
        rng: &mut R,
    ) -> Option<ControlFlow>
    where
        I: IntoIterator<Item = ProtottyInput>,
        R: Rng,
    {
        match self.state {
            AppState::Menu => {
                if let Some(menu_output) = self
                    .main_menu
                    .tick_with_mouse(inputs, &view.borders.menu.view)
                {
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

pub struct AppView {
    borders: Borders,
}

impl AppView {
    pub fn new() -> Self {
        Self {
            borders: Borders::new(),
        }
    }
}

impl View<App> for AppView {
    fn view<G: ViewGrid>(&mut self, app: &App, offset: Coord, depth: i32, grid: &mut G) {
        match app.state {
            AppState::Game | AppState::GameOver => {
                let next_piece_offset_x = self.borders.common.size(&app.tetris).x() as i32;
                self.borders.common.view(&app.tetris, offset, depth, grid);
                self.borders.next_piece.view(
                    &app.tetris,
                    Coord {
                        x: next_piece_offset_x,
                        ..offset
                    },
                    depth,
                    grid,
                );
            }
            AppState::Menu => {
                self.borders.menu.view(&app.main_menu, offset, depth, grid);
            }
            AppState::EndText => {
                DefaultRichTextView.view(&app.end_text, offset, depth, grid);
            }
        }
    }
}
