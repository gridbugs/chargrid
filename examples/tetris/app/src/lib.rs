use chargrid_component::prelude::*;
use chargrid_component_common::{
    border::{Border, BorderStyle},
    menu,
};
use rand::Rng;
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

struct TetrisComponent<R: Rng> {
    tetris: Tetris,
    rng: R,
    board_view: Border<TetrisBoardView>,
    next_piece_view: Border<TetrisNextPieceView>,
}

impl<R: Rng> TetrisComponent<R> {
    fn new(mut rng: R) -> Self {
        let BorderStyles { common, next_piece } = BorderStyles::new();
        Self {
            tetris: Tetris::new(&mut rng),
            rng,
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
    Exit,
    Pause,
    GameOver,
}

impl<R: Rng> Component for TetrisComponent<R> {
    type Output = Option<TetrisOutput>;
    type State = ();
    fn render(&self, _state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.board_view.render(&self.tetris.game_state, ctx, fb);
        self.next_piece_view.render(
            &self.tetris.game_state.next_piece,
            ctx.add_offset(Coord::new(
                self.board_view.size(&self.tetris.game_state, ctx).width() as i32,
                0,
            )),
            fb,
        );
    }
    fn update(&mut self, _state: &mut Self::State, _ctx: Ctx, event: Event) -> Self::Output {
        use input::*;
        match event {
            Event::Peek => (),
            Event::Input(input) => match input {
                Input::Keyboard(keys::ETX) => return Some(TetrisOutput::Exit),
                Input::Keyboard(keys::ESCAPE) => return Some(TetrisOutput::Pause),
                Input::Keyboard(KeyboardInput::Up) => self.tetris.input(TetrisInput::Up),
                Input::Keyboard(KeyboardInput::Down) => self.tetris.input(TetrisInput::Down),
                Input::Keyboard(KeyboardInput::Left) => self.tetris.input(TetrisInput::Left),
                Input::Keyboard(KeyboardInput::Right) => self.tetris.input(TetrisInput::Right),
                _ => (),
            },
            Event::Tick(duration) => {
                if let Some(meta) = self.tetris.tick(duration, &mut self.rng) {
                    match meta {
                        Meta::GameOver => return Some(TetrisOutput::GameOver),
                    }
                }
            }
        }
        None
    }
    fn size(&self, _state: &Self::State, ctx: Ctx) -> Size {
        let board_size = TetrisBoardView.size(&self.tetris.game_state, ctx);
        let next_piece_size = TetrisNextPieceView.size(&self.tetris.game_state.next_piece, ctx);
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

#[derive(Clone)]
enum MainMenuChoice {
    Play,
    Quit,
}

fn main_menu() -> menu::Menu<MainMenuChoice> {
    use menu::builder::*;
    menu_builder()
        .add_item(item(MainMenuChoice::Play, identifier::simple("Play!")))
        .add_item(item(MainMenuChoice::Quit, identifier::simple("Quit")))
        .build()
}

pub fn app<R: Rng>(rng: R) -> impl Component<Output = app::Output, State = ()> {
    use chargrid_component_common::control_flow::*;
    mkeither!(Ei = A | B);
    cf(main_menu())
        .and_then(|choice| match choice {
            MainMenuChoice::Play => {
                Ei::A(cf(TetrisComponent::new(rng)).map(|output| match output {
                    _ => app::Exit,
                }))
            }
            MainMenuChoice::Quit => Ei::B(val(app::Exit)),
        })
        .clear_each_frame()
}
