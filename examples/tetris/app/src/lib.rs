use chargrid::{
    border::{Border, BorderStyle},
    control_flow::*,
    core::TintDim,
    menu,
    prelude::*,
    text,
};
use rand::{Rng, SeedableRng};
use rand_isaac::Isaac64Rng;
use std::time::Duration;
use tetris::{GameState, Input as TetrisInput, Meta, Piece, PieceType, Tetris};

const BLANK_FOREGROUND_COLOUR: Rgba32 = Rgba32::new_rgb(24, 24, 24);
const FOREGROUND_COLOUR: Rgba32 = Rgba32::new_grey(255);
const BACKGROUND_COLOUR: Rgba32 = Rgba32::new_grey(0);
const BLOCK_CHAR: char = '+';
const BLANK_CHAR: char = '-';

const NEXT_PIECE_SIZE: [u32; 2] = [6, 4];

fn piece_colour(typ: PieceType) -> Rgba32 {
    use tetris::PieceType::*;
    match typ {
        L => Rgba32::new_rgb(187, 0, 0),
        J => Rgba32::new_rgb(0, 187, 0),
        S => Rgba32::new_rgb(0, 0, 187),
        Z => Rgba32::new_rgb(187, 187, 0),
        T => Rgba32::new_rgb(187, 0, 187),
        O => Rgba32::new_rgb(0, 187, 187),
        I => Rgba32::new_rgb(85, 85, 255),
    }
}

struct BorderStyles {
    common: BorderStyle,
    next_piece: BorderStyle,
}

impl BorderStyles {
    fn new() -> Self {
        let next_piece = BorderStyle {
            title: Some("next".to_string()),
            title_style: Style::default().with_foreground(Rgba32::new_grey(255)),
            background: Some(Rgba32::new_grey(127)),
            ..Default::default()
        };
        let common = BorderStyle {
            background: Some(Rgba32::new_grey(127)),
            ..Default::default()
        };
        Self { common, next_piece }
    }
}

struct TetrisState {
    rng: Isaac64Rng,
    tetris: Tetris,
}

struct TetrisComponent {
    board_view: Border<TetrisBoardView>,
    next_piece_view: Border<TetrisNextPieceView>,
}

impl TetrisComponent {
    fn new() -> Self {
        let BorderStyles { common, next_piece } = BorderStyles::new();
        Self {
            board_view: Border {
                component: TetrisBoardView,
                style: common,
            },
            next_piece_view: Border {
                component: TetrisNextPieceView,
                style: next_piece,
            },
        }
    }
}

enum TetrisOutput {
    GameOver,
}

impl Component for TetrisComponent {
    type Output = Option<TetrisOutput>;
    type State = TetrisState;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.board_view.render(&state.tetris.game_state, ctx, fb);
        self.next_piece_view.render(
            &state.tetris.game_state.next_piece,
            ctx.add_offset(Coord::new(
                self.board_view.size(&state.tetris.game_state, ctx).width() as i32,
                0,
            )),
            fb,
        );
    }
    fn update(&mut self, state: &mut Self::State, _ctx: Ctx, event: Event) -> Self::Output {
        use input::*;
        use KeyboardEvent::*;
        match event {
            Event::Peek => (),
            Event::Input(input) => match input {
                Input::Keyboard(KeyboardInput {
                    key: Key::Up,
                    event: KeyPress,
                }) => state.tetris.input(TetrisInput::Up),
                Input::Keyboard(KeyboardInput {
                    key: Key::Down,
                    event: KeyPress,
                }) => state.tetris.input(TetrisInput::Down),
                Input::Keyboard(KeyboardInput {
                    key: Key::Left,
                    event: KeyPress,
                }) => state.tetris.input(TetrisInput::Left),
                Input::Keyboard(KeyboardInput {
                    key: Key::Right,
                    event: KeyPress,
                }) => state.tetris.input(TetrisInput::Right),
                _ => (),
            },
            Event::Tick(duration) => {
                if let Some(meta) = state.tetris.tick(duration, &mut state.rng) {
                    match meta {
                        Meta::GameOver => return Some(TetrisOutput::GameOver),
                    }
                }
            }
        }
        None
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        let board_size = TetrisBoardView.size(&state.tetris.game_state, ctx);
        let next_piece_size = TetrisNextPieceView.size(&state.tetris.game_state.next_piece, ctx);
        board_size.set_width(board_size.width() + next_piece_size.width())
    }
}

struct TetrisBoardView;
struct TetrisNextPieceView;

impl Component for TetrisBoardView {
    type Output = ();
    type State = GameState;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        for (i, row) in state.board.rows.iter().enumerate() {
            for (j, cell) in row.cells.iter().enumerate() {
                let mut cell_info = RenderCell::default().with_bold(true);
                if let Some(typ) = cell.typ {
                    cell_info.character = Some(BLOCK_CHAR);
                    cell_info.style.foreground = Some(FOREGROUND_COLOUR);
                    cell_info.style.background = Some(piece_colour(typ));
                } else {
                    cell_info.character = Some(BLANK_CHAR);
                    cell_info.style.foreground = Some(BLANK_FOREGROUND_COLOUR);
                    cell_info.style.background = Some(BACKGROUND_COLOUR);
                }
                fb.set_cell_relative_to_ctx(ctx, Coord::new(j as i32, i as i32), 0, cell_info);
            }
        }
        for coord in state.piece.coords.iter().cloned() {
            let cell_info = RenderCell {
                character: Some(BLOCK_CHAR),
                style: Style {
                    bold: Some(true),
                    underline: Some(false),
                    foreground: Some(FOREGROUND_COLOUR),
                    background: Some(piece_colour(state.piece.typ)),
                },
            };
            fb.set_cell_relative_to_ctx(ctx, coord, 0, cell_info);
        }
    }
    fn update(&mut self, _state: &mut Self::State, _ctx: Ctx, _event: Event) -> Self::Output {}
    fn size(&self, state: &Self::State, _ctx: Ctx) -> Size {
        state.board.size
    }
}

impl Component for TetrisNextPieceView {
    type Output = ();
    type State = Piece;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        let offset = Coord::new(1, 0);
        for coord in state.coords.iter().cloned() {
            let cell_info = RenderCell {
                character: Some(BLOCK_CHAR),
                style: Style {
                    bold: Some(true),
                    underline: Some(false),
                    foreground: Some(FOREGROUND_COLOUR),
                    background: Some(piece_colour(state.typ)),
                },
            };
            fb.set_cell_relative_to_ctx(ctx, offset + coord, 0, cell_info);
        }
    }
    fn update(&mut self, _state: &mut Self::State, _ctx: Ctx, _event: Event) -> Self::Output {}
    fn size(&self, _state: &Self::State, _ctx: Ctx) -> Size {
        NEXT_PIECE_SIZE.into()
    }
}

#[derive(Clone, Copy)]
enum PauseMenuChoice {
    Resume,
    Restart,
    Quit,
}

fn pause_menu() -> CF<Option<PauseMenuChoice>, TetrisState> {
    use menu::builder::*;
    let BorderStyles { common, .. } = BorderStyles::new();
    menu_builder()
        .add_item(item(PauseMenuChoice::Resume, identifier::simple("Resume")))
        .add_item(item(
            PauseMenuChoice::Restart,
            identifier::simple("Restart"),
        ))
        .add_item(item(PauseMenuChoice::Quit, identifier::simple("Quit")))
        .build_cf::<Tetris>()
        .lens_state(lens!(TetrisState[tetris]: Tetris))
        .fill(Rgba32::new_grey(0))
        .border(common)
}

fn tetris() -> CF<Option<TetrisOutput>, TetrisState> {
    cf(TetrisComponent::new())
}

#[derive(Clone)]
enum PausableTetrisOutput {
    MainMenu,
    Exit,
}

fn pausable_tetris() -> CF<Option<PausableTetrisOutput>, TetrisState> {
    loop_((), |()| {
        tetris()
            .catch_escape()
            .and_then(|or_escape| match or_escape {
                Err(Escape) => pause_menu()
                    .centre()
                    .overlay_tint(tetris(), TintDim(63), 10)
                    .catch_escape()
                    .and_then(|choice| {
                        on_state(move |s: &mut TetrisState| match choice {
                            Ok(PauseMenuChoice::Resume) | Err(Escape) => LoopControl::Continue(()),
                            Ok(PauseMenuChoice::Restart) => {
                                s.tetris = Tetris::new(&mut s.rng);
                                LoopControl::Continue(())
                            }
                            Ok(PauseMenuChoice::Quit) => {
                                LoopControl::Break(PausableTetrisOutput::Exit)
                            }
                        })
                    }),
                Ok(TetrisOutput::GameOver) => cf(text::StyledString {
                    string: "YOU DIED".to_string(),
                    style: Style {
                        foreground: Some(Rgba32::new_rgb(255, 0, 0)),
                        bold: Some(true),
                        ..Default::default()
                    },
                })
                .centre()
                .ignore_state()
                .delay(Duration::from_millis(1000))
                .map(|()| LoopControl::Break(PausableTetrisOutput::MainMenu)),
            })
    })
}

#[derive(Clone)]
enum MainMenuChoice {
    Play,
    Quit,
}

fn main_menu() -> CF<Option<MainMenuChoice>, TetrisState> {
    use menu::builder::*;
    let BorderStyles { common, .. } = BorderStyles::new();
    menu_builder()
        .add_item(item(MainMenuChoice::Play, identifier::simple("Play!")))
        .add_item(item(MainMenuChoice::Quit, identifier::simple("Quit")))
        .build_cf()
        .border(common)
        .centre()
}

pub fn app<R: Rng>(mut rng: R) -> impl Component<Output = app::Output, State = ()> {
    let state = TetrisState {
        tetris: Tetris::new(&mut rng),
        rng: Isaac64Rng::from_rng(&mut rng).unwrap(),
    };
    loop_state(state, (), |()| {
        main_menu().and_then(|choice| match choice {
            MainMenuChoice::Play => on_state_then(|s: &mut TetrisState| {
                s.tetris = Tetris::new(&mut s.rng);
                pausable_tetris()
            })
            .map(|output| match output {
                PausableTetrisOutput::Exit => LoopControl::Break(app::Exit),
                PausableTetrisOutput::MainMenu => LoopControl::Continue(()),
            }),
            MainMenuChoice::Quit => val(LoopControl::Break(app::Exit)),
        })
    })
    .clear_each_frame()
    .exit_on_close()
}
