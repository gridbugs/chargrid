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
impl<'a> View<&'a Tetris> for TetrisBoardView {
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        tetris: &'a Tetris,
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        for (i, row) in tetris.game_state.board.rows.iter().enumerate() {
            for (j, cell) in row.cells.iter().enumerate() {
                let mut cell_info = ViewCell::new().with_bold(true);
                if let Some(typ) = cell.typ {
                    cell_info.character = Some(BLOCK_CHAR);
                    cell_info.style.foreground = Some(FOREGROUND_COLOUR);
                    cell_info.style.background = Some(piece_colour(typ));
                } else {
                    cell_info.character = Some(BLANK_CHAR);
                    cell_info.style.foreground = Some(BLANK_FOREGROUND_COLOUR);
                    cell_info.style.background = Some(BACKGROUND_COLOUR);
                }
                grid.set_cell_relative(
                    Coord::new(j as i32, i as i32),
                    0,
                    cell_info,
                    context,
                );
            }
        }
        for coord in tetris.game_state.piece.coords.iter().cloned() {
            let cell_info = ViewCell {
                character: Some(BLOCK_CHAR),
                style: Style {
                    bold: Some(true),
                    underline: Some(false),
                    foreground: Some(FOREGROUND_COLOUR),
                    background: Some(piece_colour(tetris.game_state.piece.typ)),
                },
            };
            grid.set_cell_relative(coord, 0, cell_info, context);
        }
    }
    fn visible_bounds<R: ViewTransformRgb24>(
        &mut self,
        tetris: &'a Tetris,
        _context: ViewContext<R>,
    ) -> Size {
        tetris.size().into()
    }
}

impl<'a> View<&'a Tetris> for TetrisNextPieceView {
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        tetris: &'a Tetris,
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        let offset = Coord::new(1, 0);
        for coord in tetris.game_state.next_piece.coords.iter().cloned() {
            let cell_info = ViewCell {
                character: Some(BLOCK_CHAR),
                style: Style {
                    bold: Some(true),
                    underline: Some(false),
                    foreground: Some(FOREGROUND_COLOUR),
                    background: Some(piece_colour(tetris.game_state.next_piece.typ)),
                },
            };
            grid.set_cell_relative(offset + coord, 0, cell_info, context);
        }
    }
    fn visible_bounds<R: ViewTransformRgb24>(
        &mut self,
        _data: &'a Tetris,
        _context: ViewContext<R>,
    ) -> Size {
        NEXT_PIECE_SIZE.into()
    }
}

struct Borders {
    common: Bordered<TetrisBoardView>,
    next_piece: Bordered<TetrisNextPieceView>,
    menu: Bordered<MenuInstanceView<MainMenuEntryView>>,
}

impl Borders {
    fn new() -> Self {
        let next_piece = Border {
            title_style: Style::new().with_foreground(colours::WHITE),
            ..Border::default_with_title("next")
        };
        let common = Border::default();
        let menu_instance_view = MenuInstanceView::new(MainMenuEntryView);
        Self {
            common: Bordered::new(TetrisBoardView, common.clone()),
            next_piece: Bordered::new(TetrisNextPieceView, next_piece),
            menu: Bordered::new(menu_instance_view, common),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum MainMenuChoice {
    Play,
    Quit,
}

struct MainMenuEntryView;

impl MenuEntryView<MainMenuChoice> for MainMenuEntryView {
    fn normal<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        choice: &MainMenuChoice,
        context: ViewContext<R>,
        grid: &mut G,
    ) -> u32 {
        let string = match choice {
            MainMenuChoice::Play => "  Play",
            MainMenuChoice::Quit => "  Quit",
        };
        StringViewSingleLine::new(Style::default())
            .view_reporting_intended_size(string, context, grid)
            .width()
    }
    fn selected<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        choice: &MainMenuChoice,
        context: ViewContext<R>,
        grid: &mut G,
    ) -> u32 {
        let base_style = Style::new().with_bold(true);
        let rich_text = match choice {
            MainMenuChoice::Play => vec![
                ("> ", base_style.with_foreground(colours::RED)),
                ("P", base_style.with_foreground(colours::YELLOW)),
                ("l", base_style.with_foreground(colours::GREEN)),
                ("a", base_style.with_foreground(colours::CYAN)),
                ("y", base_style.with_foreground(colours::BLUE)),
                ("!", base_style.with_foreground(colours::MAGENTA)),
            ],
            MainMenuChoice::Quit => vec![("> Quit", base_style)],
        };
        RichTextViewSingleLine::new()
            .view_reporting_intended_size(&rich_text, context, grid)
            .width()
    }
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
    end_text: RichTextPartOwned,
    input_buffer: VecDeque<TetrisInput>,
}

impl App {
    pub fn new<R: Rng>(rng: &mut R) -> Self {
        let main_menu = vec![MainMenuChoice::Play, MainMenuChoice::Quit];
        let main_menu = MenuInstance::new(main_menu).unwrap();
        let end_text_style = Style::default()
            .with_bold(true)
            .with_foreground(colours::RED);
        let end_text = RichTextPartOwned::new("YOU DIED".to_string(), end_text_style);
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
                        ProtottyInput::Down => {
                            self.input_buffer.push_back(TetrisInput::Down)
                        }
                        ProtottyInput::Left => {
                            self.input_buffer.push_back(TetrisInput::Left)
                        }
                        ProtottyInput::Right => {
                            self.input_buffer.push_back(TetrisInput::Right)
                        }
                        _ => (),
                    }
                }
                if let Some(meta) =
                    self.tetris.tick(self.input_buffer.drain(..), period, rng)
                {
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

impl<'a> View<&'a App> for AppView {
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        app: &'a App,
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        match app.state {
            AppState::Game | AppState::GameOver => {
                let next_piece_offset_x =
                    self.borders.common.visible_bounds(&app.tetris, context).x() as i32;
                self.borders.common.view(&app.tetris, context, grid);
                decorate(&mut self.borders.next_piece)
                    .transform_rgb24(|rgb24: Rgb24| rgb24.normalised_scalar_mul(255))
                    .view(
                        &app.tetris,
                        context.add_offset(Coord {
                            x: next_piece_offset_x,
                            y: 0,
                        }),
                        grid,
                    );
            }
            AppState::Menu => {
                self.borders.menu.view(&app.main_menu, context, grid);
            }
            AppState::EndText => {
                Aligned::centre(RichStringViewSingleLine).view(
                    &app.end_text,
                    context,
                    grid,
                );
            }
        }
    }
}
