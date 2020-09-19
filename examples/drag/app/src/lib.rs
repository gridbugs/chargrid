use chargrid::app;
use chargrid::input::*;
use chargrid::render::*;
use line_2d::{Coord, LineSegment};

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

#[derive(Default)]
pub struct AppData {
    coord: Option<Coord>,
    last_clicked_coord: Option<Coord>,
    line_type: LineType,
}

impl AppData {
    pub fn update<I: IntoIterator<Item = Input>>(&mut self, inputs: I) -> Option<app::ControlFlow> {
        for input in inputs {
            match input {
                Input::Mouse(MouseInput::MouseMove { coord, .. }) => {
                    self.coord = Some(coord);
                }
                Input::Mouse(MouseInput::MousePress { coord, button }) => {
                    self.last_clicked_coord = Some(coord);
                    self.line_type = match button {
                        MouseButton::Left => LineType::Normal,
                        MouseButton::Right => LineType::Cardinal,
                        MouseButton::Middle => LineType::Infinite,
                    }
                }
                Input::Mouse(MouseInput::MouseRelease { .. }) => {
                    self.last_clicked_coord = None;
                }
                Input::Keyboard(keys::ETX) | Input::Keyboard(keys::ESCAPE) => return Some(app::ControlFlow::Exit),
                _ => (),
            }
        }
        None
    }
}

fn draw_line<F: Frame, C: ColModify, I: IntoIterator<Item = Coord>>(frame: &mut F, iter: I, context: ViewContext<C>) {
    for coord in iter {
        if !coord.is_valid(context.size) {
            break;
        }
        frame.set_cell_relative(
            coord,
            0,
            ViewCell::new()
                .with_bold(true)
                .with_background(Rgb24::new_grey(255))
                .with_foreground(Rgb24::new(0, 187, 0))
                .with_character('.'),
            context,
        );
    }
}

impl<'a> View<&'a AppData> for AppView {
    fn view<F: Frame, C: ColModify>(&mut self, app: &'a AppData, context: ViewContext<C>, frame: &mut F) {
        let context = context.compose_col_modify(ColModifyMap(|rgb24: Rgb24| rgb24.normalised_scalar_mul(128)));
        if let (Some(last_clicked_coord), Some(coord)) = (app.last_clicked_coord, app.coord) {
            if let Ok(line) = LineSegment::try_new(last_clicked_coord, coord) {
                match app.line_type {
                    LineType::Normal => draw_line(frame, line.iter(), context),
                    LineType::Cardinal => draw_line(frame, line.cardinal_iter(), context),
                    LineType::Infinite => {
                        if line.num_steps() > 1 {
                            draw_line(frame, line.infinite_iter(), context);
                        }
                    }
                }
            }
        }
    }
}

pub struct App {
    input_buffer: Vec<Input>,
    data: AppData,
    view: AppView,
}

impl app::App for App {
    fn on_input(&mut self, input: Input) -> Option<app::ControlFlow> {
        self.input_buffer.push(input);
        None
    }
    fn on_frame<F, C>(
        &mut self,
        _since_last_frame: app::Duration,
        view_context: app::ViewContext<C>,
        frame: &mut F,
    ) -> Option<app::ControlFlow>
    where
        F: app::Frame,
        C: app::ColModify,
    {
        if let Some(app::ControlFlow::Exit) = self.data.update(self.input_buffer.drain(..)) {
            return Some(app::ControlFlow::Exit);
        }
        self.view.view(&self.data, view_context, frame);
        None
    }
}

impl Default for App {
    fn default() -> Self {
        App {
            input_buffer: Vec::new(),
            data: AppData::default(),
            view: AppView,
        }
    }
}
