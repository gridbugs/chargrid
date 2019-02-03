extern crate line_2d;
extern crate prototty;

use line_2d::{Coord, LineSegment};
use prototty::{colours, inputs, Input, MouseButton, ProtottyInput, View, ViewCell, ViewGrid};

#[derive(Debug, Clone, Copy)]
enum LineType {
    Normal,
    Cardinal,
    Infinite,
}

impl Default for LineType {
    fn default() -> Self {
        LineType::Normal
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AppView;

pub struct Quit;

#[derive(Default)]
pub struct App {
    coord: Option<Coord>,
    last_clicked_coord: Option<Coord>,
    line_type: LineType,
}

impl App {
    pub fn update<I: IntoIterator<Item = ProtottyInput>>(&mut self, inputs: I) -> Option<Quit> {
        for input in inputs {
            match input {
                Input::MouseMove { coord, .. } => {
                    self.coord = Some(coord);
                }
                Input::MousePress { coord, button } => {
                    self.last_clicked_coord = Some(coord);
                    self.line_type = match button {
                        MouseButton::Left => LineType::Normal,
                        MouseButton::Right => LineType::Cardinal,
                        MouseButton::Middle => LineType::Infinite,
                    }
                }
                Input::MouseRelease {
                    coord: _,
                    button: _,
                } => {
                    self.last_clicked_coord = None;
                }
                inputs::ETX | inputs::ESCAPE => return Some(Quit),
                _ => (),
            }
        }
        None
    }
}

fn draw_line<G: ViewGrid, I: IntoIterator<Item = Coord>>(
    grid: &mut G,
    iter: I,
    offset: Coord,
    depth: i32,
) {
    for coord in iter {
        if !coord.is_valid(grid.size()) {
            break;
        }
        grid.set_cell(
            coord + offset,
            depth,
            ViewCell::new().with_background(colours::WHITE),
        );
    }
}

impl View<App> for AppView {
    fn view<G: ViewGrid>(&mut self, app: &App, offset: Coord, depth: i32, grid: &mut G) {
        match (app.last_clicked_coord, app.coord) {
            (Some(last_clicked_coord), Some(coord)) => {
                let line = LineSegment::new(last_clicked_coord, coord);
                match app.line_type {
                    LineType::Normal => draw_line(grid, line.traverse(), offset, depth),
                    LineType::Cardinal => draw_line(grid, line.traverse_cardinal(), offset, depth),
                    LineType::Infinite => {
                        if let Ok(line) = line.try_infinite() {
                            draw_line(grid, line.traverse(), offset, depth);
                        }
                    }
                }
            }
            _ => (),
        }
    }
}
