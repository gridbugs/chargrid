use chargrid::{
    align::Align,
    border::{Border, BorderStyle},
    control_flow::*,
    core::TintDim,
    fill::Fill,
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
        match event {
            Event::Peek => (),
            Event::Input(input) => match input {
                Input::Keyboard(KeyboardInput::Up) => state.tetris.input(TetrisInput::Up),
                Input::Keyboard(KeyboardInput::Down) => state.tetris.input(TetrisInput::Down),
                Input::Keyboard(KeyboardInput::Left) => state.tetris.input(TetrisInput::Left),
                Input::Keyboard(KeyboardInput::Right) => state.tetris.input(TetrisInput::Right),
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

fn pause_menu() -> CF<impl Component<State = TetrisState, Output = Option<PauseMenuChoice>>> {
    use menu::builder::*;
    let BorderStyles { common, .. } = BorderStyles::new();
    let menu = menu_builder()
        .add_item(item(PauseMenuChoice::Resume, identifier::simple("Resume")))
        .add_item(item(
            PauseMenuChoice::Restart,
            identifier::simple("Restart"),
        ))
        .add_item(item(PauseMenuChoice::Quit, identifier::simple("Quit")))
        .build_cf::<Tetris>()
        .lens_state(mklens!(TetrisState::tetris: Tetris));
    cf(Border {
        component: Fill {
            component: menu,
            background: Rgba32::new_grey(0),
        },
        style: common,
    })
}

fn tetris() -> CF<TetrisComponent> {
    cf(TetrisComponent::new())
}

#[derive(Clone)]
enum PausableTetrisOutput {
    MainMenu,
    Exit,
}

fn pausable_tetris(
) -> CF<impl Component<Output = Option<PausableTetrisOutput>, State = TetrisState>> {
    either!(Ei = A | B);
    loop_(|| {
        tetris()
            .catch_escape()
            .and_then(|or_escape| match or_escape {
                OrEscape::Escape => Ei::A(
                    cf(Align::centre(pause_menu()))
                        .overlay(tetris(), TintDim(63), 10)
                        .catch_escape()
                        .and_then(|choice| {
                            with_state(move |s: &mut TetrisState| match choice {
                                OrEscape::Value(PauseMenuChoice::Resume) | OrEscape::Escape => {
                                    LoopControl::Continue
                                }
                                OrEscape::Value(PauseMenuChoice::Restart) => {
                                    s.tetris = Tetris::new(&mut s.rng);
                                    LoopControl::Continue
                                }
                                OrEscape::Value(PauseMenuChoice::Quit) => {
                                    LoopControl::Break(PausableTetrisOutput::Exit)
                                }
                            })
                        }),
                ),
                OrEscape::Value(TetrisOutput::GameOver) => Ei::B(
                    cf(Align::centre(text::StyledString {
                        string: "YOU DIED".to_string(),
                        style: Style {
                            foreground: Some(rgba32::rgba32_rgb(255, 0, 0)),
                            bold: Some(true),
                            ..Default::default()
                        },
                    }))
                    .ignore_state()
                    .delay(Duration::from_millis(1000))
                    .map(|()| LoopControl::Break(PausableTetrisOutput::MainMenu)),
                ),
            })
    })
}

#[derive(Clone)]
enum MainMenuChoice {
    Play,
    Quit,
}

fn main_menu() -> CF<Align<Border<menu::MenuCF<MainMenuChoice, TetrisState>>>> {
    use menu::builder::*;
    let BorderStyles { common, .. } = BorderStyles::new();
    let menu = menu_builder()
        .add_item(item(MainMenuChoice::Play, identifier::simple("Play!")))
        .add_item(item(MainMenuChoice::Quit, identifier::simple("Quit")))
        .build_cf();
    cf(Align::centre(Border {
        component: menu,
        style: common,
    }))
}

pub fn app<R: Rng>(mut rng: R) -> impl Component<Output = app::Output, State = ()> {
    let state = TetrisState {
        tetris: Tetris::new(&mut rng),
        rng: Isaac64Rng::from_rng(&mut rng).unwrap(),
    };
    either!(Ei = A | B);
    loop_state(state, || {
        main_menu().and_then(|choice| match choice {
            MainMenuChoice::Play => Ei::A(
                with_state_then(|s: &mut TetrisState| {
                    s.tetris = Tetris::new(&mut s.rng);
                    pausable_tetris()
                })
                .map(|output| match output {
                    PausableTetrisOutput::Exit => LoopControl::Break(app::Exit),
                    PausableTetrisOutput::MainMenu => LoopControl::Continue,
                }),
            ),
            MainMenuChoice::Quit => Ei::B(val(LoopControl::Break(app::Exit))),
        })
    })
    .clear_each_frame()
    .exit_on_close()
}
