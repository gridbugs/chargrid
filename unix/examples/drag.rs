extern crate line_2d;
extern crate prototty_unix;

use line_2d::{Coord, LineSegment};
use prototty_unix::prototty_input::{inputs, Input, MouseButton};
use prototty_unix::prototty_render::{colours, View, ViewCell, ViewGrid};
use prototty_unix::Context;

struct AppView;
#[derive(Default)]
struct App {
    coord: Option<Coord>,
    last_clicked_coord: Option<Coord>,
    cardinal_only: bool,
}

fn draw_line<G: ViewGrid, I: IntoIterator<Item = Coord>>(
    grid: &mut G,
    iter: I,
    offset: Coord,
    depth: i32,
) {
    for coord in iter {
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
                if app.cardinal_only {
                    draw_line(
                        grid,
                        LineSegment::new(last_clicked_coord, coord),
                        offset,
                        depth,
                    );
                } else {
                    draw_line(
                        grid,
                        LineSegment::new(last_clicked_coord, coord),
                        offset,
                        depth,
                    );
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
            Input::MouseMove(coord) => {
                app.coord = Some(coord);
            }
            Input::MousePress { coord, button } => {
                app.last_clicked_coord = Some(coord);
                match button {
                    MouseButton::Right => app.cardinal_only = true,
                    MouseButton::Left | MouseButton::Middle => app.cardinal_only = false,
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
