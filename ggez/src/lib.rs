use chargrid_input::{keys, Input, KeyboardInput, MouseButton, MouseInput, ScrollDirection};
use chargrid_runtime::{app, on_frame, on_input, Component, Coord, FrameBuffer, Rgba32, Size};
use ggez::winit;
use std::time::Instant;

const FONT_NAME_NORMAL: &'static str = "normal";
const FONT_NAME_BOLD: &'static str = "bold";

pub struct FontBytes {
    pub normal: Vec<u8>,
    pub bold: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
pub struct Dimensions<T> {
    pub width: T,
    pub height: T,
}

pub struct Config {
    pub title: String,
    pub font_bytes: FontBytes,
    pub window_dimensions_px: Dimensions<f64>,
    pub cell_dimensions_px: Dimensions<f64>,
    pub font_scale: Dimensions<f64>,
    pub underline_width_cell_ratio: f64,
    pub underline_top_offset_cell_ratio: f64,
    pub resizable: bool,
}

pub struct Context {
    config: Config,
}

struct Fonts {
    normal: ggez::graphics::FontData,
    bold: ggez::graphics::FontData,
}

struct GgezApp<C>
where
    C: 'static + Component<State = (), Output = app::Output>,
{
    chargrid_core: C,
    chargrid_frame_buffer: FrameBuffer,
    last_frame: Instant,
    font_scale: ggez::graphics::PxScale,
    underline_mesh: ggez::graphics::Mesh,
    background_mesh: ggez::graphics::Mesh,
    cell_width: f32,
    cell_height: f32,
    current_mouse_button: Option<MouseButton>,
    current_mouse_position: Coord,
    #[cfg(feature = "gamepad")]
    gamepad_id_to_integer_id: hashbrown::HashMap<ggez::event::GamepadId, u64>,
}

impl<C> GgezApp<C>
where
    C: 'static + Component<State = (), Output = app::Output>,
{
    fn convert_mouse_position(&self, x: f32, y: f32) -> Coord {
        Coord {
            x: (x / self.cell_width) as i32,
            y: (y / self.cell_height) as i32,
        }
    }

    fn convert_mouse_button(button: ggez::event::MouseButton) -> Option<MouseButton> {
        match button {
            ggez::input::mouse::MouseButton::Left => Some(MouseButton::Left),
            ggez::input::mouse::MouseButton::Right => Some(MouseButton::Right),
            ggez::input::mouse::MouseButton::Middle => Some(MouseButton::Middle),
            ggez::input::mouse::MouseButton::Other(_) => None,
        }
    }
}

impl<C> ggez::event::EventHandler<ggez::GameError> for GgezApp<C>
where
    C: 'static + Component<State = (), Output = app::Output>,
{
    fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        const DESIRED_FPS: u32 = 60;
        while ctx.time.check_update_time(DESIRED_FPS) {
            let now = Instant::now();
            if let Some(app::Exit) = on_frame(
                &mut self.chargrid_core,
                now - self.last_frame,
                &mut self.chargrid_frame_buffer,
            ) {
                ctx.request_quit();
            }
            self.last_frame = now;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        let mut canvas =
            ggez::graphics::Canvas::from_frame(ctx, ggez::graphics::Color::from([0., 0., 0., 1.]));
        for (coord, cell) in self.chargrid_frame_buffer.enumerate() {
            if cell.character != ' ' {
                let mut text = ggez::graphics::Text::new(cell.character);
                let font = if cell.bold {
                    FONT_NAME_NORMAL
                } else {
                    FONT_NAME_BOLD
                };
                text.set_font(font);
                text.set_scale(self.font_scale);
                canvas.draw(
                    &text,
                    ggez::graphics::DrawParam::new()
                        .dest([
                            coord.x as f32 * self.cell_width,
                            coord.y as f32 * self.cell_height,
                        ])
                        .color(cell.foreground.to_f32_array_01())
                        .z(1),
                );
            }
            if cell.background != Rgba32::new(0, 0, 0, 255) {
                canvas.draw(
                    &self.background_mesh,
                    ggez::graphics::DrawParam::new()
                        .dest([
                            coord.x as f32 * self.cell_width,
                            coord.y as f32 * self.cell_height,
                        ])
                        .color(cell.background.to_f32_array_01()),
                );
            }
            if cell.underline {
                canvas.draw(
                    &self.underline_mesh,
                    ggez::graphics::DrawParam::default()
                        .dest(ggez::mint::Point2 {
                            x: coord.x as f32 * self.cell_width,
                            y: coord.y as f32 * self.cell_height,
                        })
                        .color(cell.foreground.to_f32_array_01()),
                );
            }
        }
        canvas.finish(ctx).unwrap();
        ggez::timer::yield_now();
        Ok(())
    }

    fn resize_event(
        &mut self,
        _ctx: &mut ggez::Context,
        _width: f32,
        _height: f32,
    ) -> Result<(), ggez::GameError> {
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut ggez::Context,
        input: ggez::input::keyboard::KeyInput,
        _repeat: bool,
    ) -> Result<(), ggez::GameError> {
        if let Some(keycode) = input.keycode {
            use winit::event::VirtualKeyCode;
            let key_char_shift = |lower: char, upper: char| {
                KeyboardInput::Char(
                    if input.mods.contains(ggez::input::keyboard::KeyMods::SHIFT) {
                        upper
                    } else {
                        lower
                    },
                )
            };
            let input = match keycode {
                VirtualKeyCode::A => key_char_shift('a', 'A'),
                VirtualKeyCode::B => key_char_shift('b', 'B'),
                VirtualKeyCode::C => key_char_shift('c', 'C'),
                VirtualKeyCode::D => key_char_shift('d', 'D'),
                VirtualKeyCode::E => key_char_shift('e', 'E'),
                VirtualKeyCode::F => key_char_shift('f', 'F'),
                VirtualKeyCode::G => key_char_shift('g', 'G'),
                VirtualKeyCode::H => key_char_shift('h', 'H'),
                VirtualKeyCode::I => key_char_shift('i', 'I'),
                VirtualKeyCode::J => key_char_shift('j', 'J'),
                VirtualKeyCode::K => key_char_shift('k', 'K'),
                VirtualKeyCode::L => key_char_shift('l', 'L'),
                VirtualKeyCode::M => key_char_shift('m', 'M'),
                VirtualKeyCode::N => key_char_shift('n', 'N'),
                VirtualKeyCode::O => key_char_shift('o', 'O'),
                VirtualKeyCode::P => key_char_shift('p', 'P'),
                VirtualKeyCode::Q => key_char_shift('q', 'Q'),
                VirtualKeyCode::R => key_char_shift('r', 'R'),
                VirtualKeyCode::S => key_char_shift('s', 'S'),
                VirtualKeyCode::T => key_char_shift('t', 'T'),
                VirtualKeyCode::U => key_char_shift('u', 'U'),
                VirtualKeyCode::V => key_char_shift('v', 'V'),
                VirtualKeyCode::W => key_char_shift('w', 'W'),
                VirtualKeyCode::X => key_char_shift('x', 'X'),
                VirtualKeyCode::Y => key_char_shift('y', 'Y'),
                VirtualKeyCode::Z => key_char_shift('z', 'Z'),
                VirtualKeyCode::Key1 => KeyboardInput::Char('1'),
                VirtualKeyCode::Key2 => KeyboardInput::Char('2'),
                VirtualKeyCode::Key3 => KeyboardInput::Char('3'),
                VirtualKeyCode::Key4 => KeyboardInput::Char('4'),
                VirtualKeyCode::Key5 => KeyboardInput::Char('5'),
                VirtualKeyCode::Key6 => KeyboardInput::Char('6'),
                VirtualKeyCode::Key7 => KeyboardInput::Char('7'),
                VirtualKeyCode::Key8 => KeyboardInput::Char('8'),
                VirtualKeyCode::Key9 => KeyboardInput::Char('9'),
                VirtualKeyCode::Key0 => KeyboardInput::Char('0'),
                VirtualKeyCode::Numpad1 => KeyboardInput::Char('1'),
                VirtualKeyCode::Numpad2 => KeyboardInput::Char('2'),
                VirtualKeyCode::Numpad3 => KeyboardInput::Char('3'),
                VirtualKeyCode::Numpad4 => KeyboardInput::Char('4'),
                VirtualKeyCode::Numpad5 => KeyboardInput::Char('5'),
                VirtualKeyCode::Numpad6 => KeyboardInput::Char('6'),
                VirtualKeyCode::Numpad7 => KeyboardInput::Char('7'),
                VirtualKeyCode::Numpad8 => KeyboardInput::Char('8'),
                VirtualKeyCode::Numpad9 => KeyboardInput::Char('9'),
                VirtualKeyCode::Numpad0 => KeyboardInput::Char('0'),
                VirtualKeyCode::F1 => KeyboardInput::Function(1),
                VirtualKeyCode::F2 => KeyboardInput::Function(2),
                VirtualKeyCode::F3 => KeyboardInput::Function(3),
                VirtualKeyCode::F4 => KeyboardInput::Function(4),
                VirtualKeyCode::F5 => KeyboardInput::Function(5),
                VirtualKeyCode::F6 => KeyboardInput::Function(6),
                VirtualKeyCode::F7 => KeyboardInput::Function(7),
                VirtualKeyCode::F8 => KeyboardInput::Function(8),
                VirtualKeyCode::F9 => KeyboardInput::Function(9),
                VirtualKeyCode::F10 => KeyboardInput::Function(10),
                VirtualKeyCode::F11 => KeyboardInput::Function(11),
                VirtualKeyCode::F12 => KeyboardInput::Function(12),
                VirtualKeyCode::F13 => KeyboardInput::Function(13),
                VirtualKeyCode::F14 => KeyboardInput::Function(14),
                VirtualKeyCode::F15 => KeyboardInput::Function(15),
                VirtualKeyCode::F16 => KeyboardInput::Function(16),
                VirtualKeyCode::F17 => KeyboardInput::Function(17),
                VirtualKeyCode::F18 => KeyboardInput::Function(18),
                VirtualKeyCode::F19 => KeyboardInput::Function(19),
                VirtualKeyCode::F20 => KeyboardInput::Function(20),
                VirtualKeyCode::F21 => KeyboardInput::Function(21),
                VirtualKeyCode::F22 => KeyboardInput::Function(22),
                VirtualKeyCode::F23 => KeyboardInput::Function(23),
                VirtualKeyCode::F24 => KeyboardInput::Function(24),
                VirtualKeyCode::At => KeyboardInput::Char('@'),
                VirtualKeyCode::Plus => KeyboardInput::Char('+'),
                VirtualKeyCode::Minus => KeyboardInput::Char('-'),
                VirtualKeyCode::Equals => key_char_shift('=', '+'),
                VirtualKeyCode::Backslash => KeyboardInput::Char('\\'),
                VirtualKeyCode::Grave => KeyboardInput::Char('`'),
                VirtualKeyCode::Apostrophe => KeyboardInput::Char('\''),
                VirtualKeyCode::LBracket => KeyboardInput::Char('['),
                VirtualKeyCode::RBracket => KeyboardInput::Char(']'),
                VirtualKeyCode::Period => KeyboardInput::Char('.'),
                VirtualKeyCode::Comma => KeyboardInput::Char(','),
                VirtualKeyCode::Slash => KeyboardInput::Char('/'),
                VirtualKeyCode::NumpadAdd => KeyboardInput::Char('+'),
                VirtualKeyCode::NumpadSubtract => KeyboardInput::Char('-'),
                VirtualKeyCode::NumpadMultiply => KeyboardInput::Char('*'),
                VirtualKeyCode::NumpadDivide => KeyboardInput::Char('/'),
                VirtualKeyCode::PageUp => KeyboardInput::PageUp,
                VirtualKeyCode::PageDown => KeyboardInput::PageDown,
                VirtualKeyCode::Home => KeyboardInput::Home,
                VirtualKeyCode::End => KeyboardInput::End,
                VirtualKeyCode::Up => KeyboardInput::Up,
                VirtualKeyCode::Down => KeyboardInput::Down,
                VirtualKeyCode::Left => KeyboardInput::Left,
                VirtualKeyCode::Right => KeyboardInput::Right,
                VirtualKeyCode::Return => keys::RETURN,
                VirtualKeyCode::Escape => keys::ESCAPE,
                VirtualKeyCode::Space => KeyboardInput::Char(' '),
                other => {
                    log::warn!("Unhandled input: {:?}", other);
                    return Ok(());
                }
            };
            if let Some(app::Exit) = on_input(
                &mut self.chargrid_core,
                Input::Keyboard(input),
                &self.chargrid_frame_buffer,
            ) {
                ctx.request_quit();
            }
        }
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut ggez::Context,
        button: ggez::event::MouseButton,
        x: f32,
        y: f32,
    ) -> Result<(), ggez::GameError> {
        if let Some(button) = Self::convert_mouse_button(button) {
            self.current_mouse_button = Some(button);
            let coord = self.convert_mouse_position(x, y);
            self.current_mouse_position = coord;
            let input = MouseInput::MousePress { button, coord };
            if let Some(app::Exit) = on_input(
                &mut self.chargrid_core,
                Input::Mouse(input),
                &self.chargrid_frame_buffer,
            ) {
                ctx.request_quit();
            }
        }
        Ok(())
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut ggez::Context,
        button: ggez::event::MouseButton,
        x: f32,
        y: f32,
    ) -> Result<(), ggez::GameError> {
        if let Some(button) = Self::convert_mouse_button(button) {
            self.current_mouse_button = None;
            let coord = self.convert_mouse_position(x, y);
            self.current_mouse_position = coord;
            let input = MouseInput::MouseRelease {
                button: Ok(button),
                coord,
            };
            if let Some(app::Exit) = on_input(
                &mut self.chargrid_core,
                Input::Mouse(input),
                &self.chargrid_frame_buffer,
            ) {
                ctx.request_quit();
            }
        }
        Ok(())
    }

    fn mouse_motion_event(
        &mut self,
        ctx: &mut ggez::Context,
        x: f32,
        y: f32,
        _dx: f32,
        _dy: f32,
    ) -> Result<(), ggez::GameError> {
        let coord = self.convert_mouse_position(x, y);
        self.current_mouse_position = coord;
        let input = MouseInput::MouseMove {
            coord,
            button: self.current_mouse_button,
        };
        if let Some(app::Exit) = on_input(
            &mut self.chargrid_core,
            Input::Mouse(input),
            &self.chargrid_frame_buffer,
        ) {
            ctx.request_quit();
        }
        Ok(())
    }

    fn mouse_wheel_event(
        &mut self,
        ctx: &mut ggez::Context,
        x: f32,
        y: f32,
    ) -> Result<(), ggez::GameError> {
        let mut handle = |direction| {
            let coord = self.current_mouse_position;
            let input = MouseInput::MouseScroll { direction, coord };
            if let Some(app::Exit) = on_input(
                &mut self.chargrid_core,
                Input::Mouse(input),
                &self.chargrid_frame_buffer,
            ) {
                ctx.request_quit();
            }
        };
        if x > 0.0 {
            handle(ScrollDirection::Right);
        }
        if x < 0.0 {
            handle(ScrollDirection::Left);
        }
        if y > 0.0 {
            handle(ScrollDirection::Up);
        }
        if y < 0.0 {
            handle(ScrollDirection::Down);
        }
        Ok(())
    }

    fn quit_event(&mut self, _ctx: &mut ggez::Context) -> Result<bool, ggez::GameError> {
        Ok(false)
    }

    #[cfg(feature = "gamepad")]
    fn gamepad_button_down_event(
        &mut self,
        ctx: &mut ggez::Context,
        btn: ggez::event::Button,
        id: ggez::event::GamepadId,
    ) -> Result<(), ggez::GameError> {
        use chargrid_input::{GamepadButton, GamepadInput};
        let num_gamepad_ids = self.gamepad_id_to_integer_id.len() as u64;
        let &mut integer_id = self
            .gamepad_id_to_integer_id
            .entry(id)
            .or_insert(num_gamepad_ids);
        let button = match btn {
            ggez::event::Button::DPadUp => GamepadButton::DPadUp,
            ggez::event::Button::DPadRight => GamepadButton::DPadRight,
            ggez::event::Button::DPadDown => GamepadButton::DPadDown,
            ggez::event::Button::DPadLeft => GamepadButton::DPadLeft,
            ggez::event::Button::North => GamepadButton::North,
            ggez::event::Button::East => GamepadButton::East,
            ggez::event::Button::South => GamepadButton::South,
            ggez::event::Button::West => GamepadButton::West,
            ggez::event::Button::Start => GamepadButton::Start,
            ggez::event::Button::Select => GamepadButton::Select,
            ggez::event::Button::LeftTrigger => GamepadButton::LeftBumper,
            ggez::event::Button::RightTrigger => GamepadButton::RightBumper,
            other => {
                log::warn!("Unhandled input: {:?}", other);
                return Ok(());
            }
        };
        let input = GamepadInput {
            button,
            id: integer_id,
        };
        if let Some(app::Exit) = on_input(
            &mut self.chargrid_core,
            Input::Gamepad(input),
            &self.chargrid_frame_buffer,
        ) {
            ctx.request_quit();
        }
        Ok(())
    }
}

impl Context {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn run<C>(self, component: C) -> !
    where
        C: 'static + Component<State = (), Output = app::Output>,
    {
        let Self { config } = self;
        let grid_size = Size::new(
            (config.window_dimensions_px.width as f64 / config.cell_dimensions_px.width) as u32,
            (config.window_dimensions_px.height as f64 / config.cell_dimensions_px.height) as u32,
        );
        let chargrid_frame_buffer = FrameBuffer::new(grid_size);
        let (mut ctx, events_loop) =
            ggez::ContextBuilder::new(config.title.as_str(), "chargrid_ggez")
                .window_setup(ggez::conf::WindowSetup::default().title(config.title.as_str()))
                .window_mode(
                    ggez::conf::WindowMode::default()
                        .dimensions(
                            config.window_dimensions_px.width as f32,
                            config.window_dimensions_px.height as f32,
                        )
                        .resizable(config.resizable),
                )
                .build()
                .expect("failed to initialize ggez");
        let fonts = Fonts {
            normal: ggez::graphics::FontData::from_vec(config.font_bytes.normal.to_vec())
                .expect("failed to load normal font"),
            bold: ggez::graphics::FontData::from_vec(config.font_bytes.bold.to_vec())
                .expect("failed to load bold font"),
        };
        let underline_mesh = {
            let underline_mid_cell_ratio =
                config.underline_top_offset_cell_ratio + config.underline_width_cell_ratio / 2.0;
            let underline_cell_position =
                (underline_mid_cell_ratio * config.cell_dimensions_px.height) as f32;
            let underline_width =
                (config.underline_width_cell_ratio * config.cell_dimensions_px.height) as f32;
            let points = [
                ggez::mint::Point2 {
                    x: 0.0,
                    y: underline_cell_position,
                },
                ggez::mint::Point2 {
                    x: config.cell_dimensions_px.width as f32,
                    y: underline_cell_position,
                },
            ];
            let mesh = ggez::graphics::Mesh::new_line(
                &mut ctx,
                &points,
                underline_width,
                [1., 1., 1., 1.].into(),
            )
            .expect("failed to build mesh for underline");
            mesh
        };
        let background_mesh = {
            let rect = ggez::graphics::Rect {
                x: 0.0,
                y: 0.0,
                w: config.cell_dimensions_px.width as f32,
                h: config.cell_dimensions_px.height as f32,
            };
            let mesh = ggez::graphics::Mesh::new_rectangle(
                &mut ctx,
                ggez::graphics::DrawMode::fill(),
                rect,
                [1., 1., 1., 1.].into(),
            )
            .expect("failed to build mesh for background");
            mesh
        };
        ctx.gfx.add_font(FONT_NAME_NORMAL, fonts.normal);
        ctx.gfx.add_font(FONT_NAME_BOLD, fonts.bold);
        ggez::event::run(
            ctx,
            events_loop,
            GgezApp {
                chargrid_core: component,
                chargrid_frame_buffer,
                last_frame: Instant::now(),
                font_scale: ggez::graphics::PxScale {
                    x: config.font_scale.width as f32,
                    y: config.font_scale.height as f32,
                },
                underline_mesh,
                background_mesh,
                cell_width: config.cell_dimensions_px.width as f32,
                cell_height: config.cell_dimensions_px.height as f32,
                current_mouse_button: None,
                current_mouse_position: Coord::new(0, 0),
                #[cfg(feature = "gamepad")]
                gamepad_id_to_integer_id: hashbrown::HashMap::default(),
            },
        )
    }
}
