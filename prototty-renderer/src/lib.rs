extern crate prototty_elements;
extern crate prototty;
extern crate tetris;
extern crate ansi_colour;
extern crate cgmath;

use cgmath::Vector2;
use prototty::{View, ViewGrid, ViewSize};
use prototty_elements::elements::*;
use ansi_colour::{colours, Colour};

const BLANK_COLOUR: Colour = colours::DARK_GREY;
const FOREGROUND_COLOUR: Colour = colours::DARK_GREY;
const BORDER_COLOUR: Colour = colours::WHITE;
const BORDER_BACKGROUND: Colour = colours::BLACK;
const BLOCK_CHAR: char = '▯';

pub struct Model {
    game_canvas: Border<Canvas>,
    piece_canvas: Border<Canvas>,
}

impl Model {
    pub fn new(width: u16, height: u16) -> Self {
        let mut model = Self {
            game_canvas: Border::new(Canvas::new((width, height))),
            piece_canvas: Border::new(Canvas::new((6, 4))),
        };

        model.piece_canvas.title = Some("next".to_string());
        model.piece_canvas.foreground_colour = BORDER_COLOUR;
        model.piece_canvas.background_colour = BORDER_BACKGROUND;
        model.game_canvas.foreground_colour = BORDER_COLOUR;
        model.game_canvas.background_colour = BORDER_BACKGROUND;

        model
    }
}

impl View for Model {
    fn view<G: ViewGrid>(&self, offset: Vector2<i16>, depth: i16, grid: &mut G) {
        self.game_canvas.view(offset + Vector2::new(1, 1), depth, grid);
        let piece_offset = Vector2::new(self.game_canvas.size().x + 1, 1).cast().unwrap();
        self.piece_canvas.view(offset + piece_offset, depth, grid);
    }
}

impl Model {
    pub fn render(&mut self, tetris: &tetris::Tetris) {
        self.render_board(tetris);
        self.render_next(tetris);
    }

    fn piece_colour(typ: tetris::PieceType) -> ansi_colour::Colour {
        use tetris::PieceType::*;
        use ansi_colour::colours;
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

    fn render_board(&mut self, tetris: &tetris::Tetris) {
        for (coord, canvas_cell) in self.game_canvas.child.enumerate_mut() {
            let board_cell = tetris.game_state.board.get(coord).unwrap();
            if let Some(typ) = board_cell.typ {
                canvas_cell.background_colour = Self::piece_colour(typ);
                canvas_cell.character = BLOCK_CHAR;
                canvas_cell.foreground_colour = FOREGROUND_COLOUR;
            } else {
                canvas_cell.character = ' ';
                canvas_cell.background_colour = BLANK_COLOUR;
            }
        }
        for coord in tetris.game_state.piece.coords.iter().cloned() {
            if let Some(canvas_cell) = self.game_canvas.child.get_mut(coord) {
                canvas_cell.background_colour = Self::piece_colour(tetris.game_state.piece.typ);
                canvas_cell.foreground_colour = FOREGROUND_COLOUR;
                canvas_cell.character = BLOCK_CHAR;
            }
        }
    }

    fn render_next(&mut self, tetris: &tetris::Tetris) {
        for cell in self.piece_canvas.child.iter_mut() {
            cell.character = ' ';
            cell.foreground_colour = BLANK_COLOUR;
            cell.background_colour = BLANK_COLOUR;
        }
        for coord in tetris.game_state.next_piece.coords.iter().cloned() {
            if let Some(cell) = self.piece_canvas.child.get_mut(coord + Vector2::new(1, 0)) {
                cell.character = BLOCK_CHAR;
                cell.foreground_colour = FOREGROUND_COLOUR;
                cell.background_colour = Self::piece_colour(tetris.game_state.next_piece.typ);
            }
        }
    }
}
