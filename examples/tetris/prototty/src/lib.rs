extern crate prototty;
extern crate rand;
extern crate tetris;

use prototty::decorator::*;
use prototty::input::{keys, Input, KeyboardInput};
use prototty::menu::*;
use prototty::render::*;
use prototty::text::*;
use rand::Rng;
use std::collections::VecDeque;
use std::time::Duration;
use tetris::{Input as TetrisInput, Meta, PieceType, Tetris};

const BLANK_FOREGROUND_COLOUR: Rgb24 = Rgb24::new(24, 24, 24);
const FOREGROUND_COLOUR: Rgb24 = Rgb24::new_grey(255);
const BACKGROUND_COLOUR: Rgb24 = Rgb24::new_grey(0);
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
        L => Rgb24::new(187, 0, 0),
        ReverseL => Rgb24::new(0, 187, 0),
        S => Rgb24::new(0, 0, 187),
        Z => Rgb24::new(187, 187, 0),
        T => Rgb24::new(187, 0, 187),
        Square => Rgb24::new(0, 187, 187),
        Line => Rgb24::new(85, 85, 255),
    }
}
impl<'a> View<&'a Tetris> for TetrisBoardView {
    fn view<F: Frame, C: ColModify>(&mut self, tetris: &'a Tetris, context: ViewContext<C>, frame: &mut F) {
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
                frame.set_cell_relative(Coord::new(j as i32, i as i32), 0, cell_info, context);
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
            frame.set_cell_relative(coord, 0, cell_info, context);
        }
    }
    fn size<C: ColModify>(&mut self, tetris: &'a Tetris, _context: ViewContext<C>) -> Size {
        tetris.size().into()
    }
}

impl<'a> View<&'a Tetris> for TetrisNextPieceView {
    fn view<F: Frame, C: ColModify>(&mut self, tetris: &'a Tetris, context: ViewContext<C>, frame: &mut F) {
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
            frame.set_cell_relative(offset + coord, 0, cell_info, context);
        }
    }
    fn size<C: ColModify>(&mut self, _data: &'a Tetris, _context: ViewContext<C>) -> Size {
        NEXT_PIECE_SIZE.into()
    }
}

struct BorderStyles {
    common: BorderStyle,
    next_piece: BorderStyle,
}

impl BorderStyles {
    pub fn new() -> Self {
        let next_piece = BorderStyle {
            title_style: Style::new().with_foreground(Rgb24::new_grey(255)),
            ..BorderStyle::default_with_title("next")
        };
        let common = BorderStyle::default();
        Self { common, next_piece }
    }
}

#[derive(Debug, Clone, Copy)]
enum MainMenuChoice {
    Play,
    Quit,
}

struct MainMenuEntryView;

impl MenuEntryView<MainMenuChoice> for MainMenuEntryView {
    fn normal<F: Frame, C: ColModify>(
        &mut self,
        choice: &MainMenuChoice,
        context: ViewContext<C>,
        frame: &mut F,
    ) -> u32 {
        let string = match choice {
            MainMenuChoice::Play => "  Play",
            MainMenuChoice::Quit => "  Quit",
        };
        StringViewSingleLine::new(Style::default())
            .view_size(string, context, frame)
            .width()
    }
    fn selected<F: Frame, C: ColModify>(
        &mut self,
        choice: &MainMenuChoice,
        context: ViewContext<C>,
        frame: &mut F,
    ) -> u32 {
        let base_style = Style::new().with_bold(true).with_underline(true);
        let rich_text = match choice {
            MainMenuChoice::Play => vec![
                ("> ", base_style.with_foreground(Rgb24::new(187, 0, 0))),
                ("P", base_style.with_foreground(Rgb24::new(187, 187, 0))),
                ("l", base_style.with_foreground(Rgb24::new(0, 187, 0))),
                ("a", base_style.with_foreground(Rgb24::new(0, 187, 187))),
                ("y", base_style.with_foreground(Rgb24::new(0, 0, 187))),
                ("!", base_style.with_foreground(Rgb24::new(187, 0, 187))),
            ],
            MainMenuChoice::Quit => vec![("> Quit", base_style)],
        };
        RichTextViewSingleLine::new()
            .view_size(&rich_text, context, frame)
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
    border_styles: BorderStyles,
}

impl App {
    pub fn new<R: Rng>(rng: &mut R) -> Self {
        let main_menu = vec![MainMenuChoice::Play, MainMenuChoice::Quit];
        let main_menu = MenuInstance::new(main_menu).unwrap();
        let end_text_style = Style::default().with_bold(true).with_foreground(Rgb24::new(187, 0, 0));
        let end_text = RichTextPartOwned::new("YOU DIED".to_string(), end_text_style);
        Self {
            main_menu,
            state: AppState::Menu,
            timeout: Timeout::zero(),
            tetris: Tetris::new(rng),
            end_text,
            input_buffer: VecDeque::with_capacity(INPUT_BUFFER_SIZE),
            border_styles: BorderStyles::new(),
        }
    }

    pub fn tick<I, R>(&mut self, inputs: I, period: Duration, view: &AppView, rng: &mut R) -> Option<ControlFlow>
    where
        I: IntoIterator<Item = Input>,
        R: Rng,
    {
        match self.state {
            AppState::Menu => {
                for input in inputs {
                    if let Some(menu_output) = self.main_menu.choose_or_quit(&view.menu_instance_view, input) {
                        match menu_output {
                            Err(Quit) => return Some(ControlFlow::Exit),
                            Ok(selection) => match selection {
                                MainMenuChoice::Quit => return Some(ControlFlow::Exit),
                                MainMenuChoice::Play => {
                                    self.state = AppState::Game;
                                }
                            },
                        }
                    }
                }
            }
            AppState::Game => {
                for input in inputs {
                    match input {
                        Input::Keyboard(keys::ETX) => return Some(ControlFlow::Exit),
                        Input::Keyboard(keys::ESCAPE) => {
                            self.state = AppState::Menu;
                        }
                        Input::Keyboard(KeyboardInput::Up) => self.input_buffer.push_back(TetrisInput::Up),
                        Input::Keyboard(KeyboardInput::Down) => self.input_buffer.push_back(TetrisInput::Down),
                        Input::Keyboard(KeyboardInput::Left) => self.input_buffer.push_back(TetrisInput::Left),
                        Input::Keyboard(KeyboardInput::Right) => self.input_buffer.push_back(TetrisInput::Right),
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
    menu_instance_view: MenuInstanceView<MainMenuEntryView>,
    board: TetrisBoardView,
    next_piece: TetrisNextPieceView,
}

impl AppView {
    pub fn new() -> Self {
        Self {
            menu_instance_view: MenuInstanceView::new(MainMenuEntryView),
            board: TetrisBoardView,
            next_piece: TetrisNextPieceView,
        }
    }
}

impl<'a> View<&'a App> for AppView {
    fn view<F: Frame, C: ColModify>(&mut self, app: &'a App, context: ViewContext<C>, frame: &mut F) {
        match app.state {
            AppState::Game | AppState::GameOver => {
                let mut view = BorderView {
                    style: &app.border_styles.common,
                    view: &mut self.board,
                };
                let next_piece_offset_x = view.view_size(&app.tetris, context, frame).x() as i32;
                ColModifyView {
                    col_modify: |rgb24: Rgb24| rgb24.normalised_scalar_mul(255),
                    view: BorderView {
                        style: &app.border_styles.next_piece,
                        view: BoundView {
                            size: Size::new(6, 4),
                            view: &mut self.next_piece,
                        },
                    },
                }
                .view(
                    &app.tetris,
                    context.add_offset(Coord::new(next_piece_offset_x, 0)),
                    frame,
                );;
            }
            AppState::Menu => {
                let mut v = BorderView {
                    style: &app.border_styles.common,
                    view: BoundView {
                        size: Size::new_u16(8, 2),
                        view: &mut self.menu_instance_view,
                    },
                };
                v.view(&app.main_menu, context, frame);
            }
            AppState::EndText => {
                AlignView {
                    alignment: Alignment::centre(),
                    view: RichStringViewSingleLine,
                }
                .view(&app.end_text, context, frame);
            }
        }
    }
}
