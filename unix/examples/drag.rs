extern crate line_2d;
extern crate prototty_unix;

use line_2d::{Coord, LineSegment};
use prototty_unix::prototty_input::{inputs, Input, MouseButton};
use prototty_unix::prototty_render::{colours, View, ViewCell, ViewGrid};
use prototty_unix::Context;

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

struct AppView;

#[derive(Default)]
struct App {
    coord: Option<Coord>,
    last_clicked_coord: Option<Coord>,
    line_type: LineType,
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

fn main() {
    let mut context = Context::new().unwrap();
    let mut app = App::default();
    loop {
        match context.wait_input().unwrap() {
            Input::MouseMove { coord, .. } => {
                app.coord = Some(coord);
            }
            Input::MousePress { coord, button } => {
                app.last_clicked_coord = Some(coord);
                app.line_type = match button {
                    MouseButton::Left => LineType::Normal,
                    MouseButton::Right => LineType::Cardinal,
                    MouseButton::Middle => LineType::Infinite,
                }
            }
            Input::MouseRelease {
                coord: _,
                button: _,
            } => {
                app.last_clicked_coord = None;
            }
            inputs::ETX => break,
            _ => (),
        }
        context.render(&mut AppView, &app).unwrap();
    }
}
