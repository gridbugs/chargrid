extern crate line_2d;
extern crate prototty;

use line_2d::{Coord, LineSegment};
use prototty::*;

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
    pub fn update<I: IntoIterator<Item = ProtottyInput>>(
        &mut self,
        inputs: I,
    ) -> Option<Quit> {
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

fn draw_line<G: ViewGrid, R: ViewTransformRgb24, I: IntoIterator<Item = Coord>>(
    grid: &mut G,
    iter: I,
    context: ViewContext<R>,
) {
    for coord in iter {
        if !coord.is_valid(grid.size()) {
            break;
        }
        grid.set_cell_relative(
            coord,
            0,
            ViewCell::new()
                .with_bold(true)
                .with_background(colours::WHITE)
                .with_foreground(colours::BLUE)
                .with_character('.'),
            context,
        );
    }
}

impl<'a> View<&'a App> for AppView {
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        app: &'a App,
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        let context = context
            .compose_transform_rgb24(|rgb24: Rgb24| rgb24.normalised_scalar_mul(128));
        match (app.last_clicked_coord, app.coord) {
            (Some(last_clicked_coord), Some(coord)) => {
                let line = LineSegment::new(last_clicked_coord, coord);
                match app.line_type {
                    LineType::Normal => draw_line(grid, line.traverse(), context),
                    LineType::Cardinal => {
                        draw_line(grid, line.traverse_cardinal(), context)
                    }
                    LineType::Infinite => {
                        if let Ok(line) = line.try_infinite() {
                            draw_line(grid, line.traverse(), context);
                        }
                    }
                }
            }
            _ => (),
        }
    }
}
