use chargrid_component_runtime::{
    on_frame, on_input, Component, ControlFlow, Coord, FrameBuffer, Rgba32, Size,
};
pub use chargrid_graphical_common::*;
use chargrid_input::{keys, Input, KeyboardInput, MouseButton, MouseInput, ScrollDirection};
use std::time::Instant;

pub struct Context {
    config: Config,
}

struct Fonts {
    normal: ggez::graphics::Font,
    bold: ggez::graphics::Font,
}

struct GgezApp<C>
where
    C: 'static + Component<State = (), Output = Option<ControlFlow>>,
{
    fonts: Fonts,
    chargrid_component: C,
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
    C: 'static + Component<State = (), Output = Option<ControlFlow>>,
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
    C: 'static + Component<State = (), Output = Option<ControlFlow>>,
{
    fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        const DESIRED_FPS: u32 = 60;
        while ggez::timer::check_update_time(ctx, DESIRED_FPS) {}
        let now = Instant::now();
        if let Some(ControlFlow::Exit) = on_frame(
            &mut self.chargrid_component,
            now - self.last_frame,
            &mut self.chargrid_frame_buffer,
        ) {
            ggez::event::quit(ctx);
        }

        self.last_frame = now;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        ggez::graphics::clear(ctx, [0., 0., 0., 1.0].into());
        for (coord, cell) in self.chargrid_frame_buffer.enumerate() {
            if cell.character != ' ' {
                let mut text = ggez::graphics::Text::new(cell.character);
                let font = if cell.bold {
                    self.fonts.bold
                } else {
                    self.fonts.normal
                };
                text.set_font(font, self.font_scale);
                ggez::graphics::queue_text(
                    ctx,
                    &text,
                    ggez::mint::Point2 {
                        x: coord.x as f32 * self.cell_width,
                        y: coord.y as f32 * self.cell_height,
                    },
                    Some(cell.foreground.to_f32_array_01().into()),
                );
            }
            if cell.background != Rgba32::new(0, 0, 0, 255) {
                ggez::graphics::draw(
                    ctx,
                    &self.background_mesh,
                    ggez::graphics::DrawParam::default()
                        .dest(ggez::mint::Point2 {
                            x: coord.x as f32 * self.cell_width,
                            y: coord.y as f32 * self.cell_height,
                        })
                        .color(cell.background.to_f32_array_01().into()),
                )
                .expect("failed to draw background");
            }
            if cell.underline {
                ggez::graphics::draw(
                    ctx,
                    &self.underline_mesh,
                    ggez::graphics::DrawParam::default()
                        .dest(ggez::mint::Point2 {
                            x: coord.x as f32 * self.cell_width,
                            y: coord.y as f32 * self.cell_height,
                        })
                        .color(cell.foreground.to_f32_array_01().into()),
                )
                .expect("failed to draw underline");
            }
        }
        ggez::graphics::draw_queued_text(
            ctx,
            ggez::graphics::DrawParam::default(),
            None,
            ggez::graphics::FilterMode::Linear,
        )?;
        ggez::graphics::present(ctx)?;
        ggez::timer::yield_now();
        Ok(())
    }

    fn resize_event(&mut self, ctx: &mut ggez::Context, width: f32, height: f32) {
        ggez::graphics::set_screen_coordinates(
            ctx,
            ggez::graphics::Rect::new(0.0, 0.0, width, height),
        )
        .unwrap();
    }

    fn key_down_event(
        &mut self,
        ctx: &mut ggez::Context,
        keycode: ggez::input::keyboard::KeyCode,
        keymods: ggez::input::keyboard::KeyMods,
        _repeat: bool,
    ) {
        let key_char_shift = |lower: char, upper: char| {
            KeyboardInput::Char(if keymods.contains(ggez::input::keyboard::KeyMods::SHIFT) {
                upper
            } else {
                lower
            })
        };
        let input = match keycode {
            ggez::event::KeyCode::A => key_char_shift('a', 'A'),
            ggez::event::KeyCode::B => key_char_shift('b', 'B'),
            ggez::event::KeyCode::C => key_char_shift('c', 'C'),
            ggez::event::KeyCode::D => key_char_shift('d', 'D'),
            ggez::event::KeyCode::E => key_char_shift('e', 'E'),
            ggez::event::KeyCode::F => key_char_shift('f', 'F'),
            ggez::event::KeyCode::G => key_char_shift('g', 'G'),
            ggez::event::KeyCode::H => key_char_shift('h', 'H'),
            ggez::event::KeyCode::I => key_char_shift('i', 'I'),
            ggez::event::KeyCode::J => key_char_shift('j', 'J'),
            ggez::event::KeyCode::K => key_char_shift('k', 'K'),
            ggez::event::KeyCode::L => key_char_shift('l', 'L'),
            ggez::event::KeyCode::M => key_char_shift('m', 'M'),
            ggez::event::KeyCode::N => key_char_shift('n', 'N'),
            ggez::event::KeyCode::O => key_char_shift('o', 'O'),
            ggez::event::KeyCode::P => key_char_shift('p', 'P'),
            ggez::event::KeyCode::Q => key_char_shift('q', 'Q'),
            ggez::event::KeyCode::R => key_char_shift('r', 'R'),
            ggez::event::KeyCode::S => key_char_shift('s', 'S'),
            ggez::event::KeyCode::T => key_char_shift('t', 'T'),
            ggez::event::KeyCode::U => key_char_shift('u', 'U'),
            ggez::event::KeyCode::V => key_char_shift('v', 'V'),
            ggez::event::KeyCode::W => key_char_shift('w', 'W'),
            ggez::event::KeyCode::X => key_char_shift('x', 'X'),
            ggez::event::KeyCode::Y => key_char_shift('y', 'Y'),
            ggez::event::KeyCode::Z => key_char_shift('z', 'Z'),
            ggez::event::KeyCode::Key1 => KeyboardInput::Char('1'),
            ggez::event::KeyCode::Key2 => KeyboardInput::Char('2'),
            ggez::event::KeyCode::Key3 => KeyboardInput::Char('3'),
            ggez::event::KeyCode::Key4 => KeyboardInput::Char('4'),
            ggez::event::KeyCode::Key5 => KeyboardInput::Char('5'),
            ggez::event::KeyCode::Key6 => KeyboardInput::Char('6'),
            ggez::event::KeyCode::Key7 => KeyboardInput::Char('7'),
            ggez::event::KeyCode::Key8 => KeyboardInput::Char('8'),
            ggez::event::KeyCode::Key9 => KeyboardInput::Char('9'),
            ggez::event::KeyCode::Key0 => KeyboardInput::Char('0'),
            ggez::event::KeyCode::Numpad1 => KeyboardInput::Char('1'),
            ggez::event::KeyCode::Numpad2 => KeyboardInput::Char('2'),
            ggez::event::KeyCode::Numpad3 => KeyboardInput::Char('3'),
            ggez::event::KeyCode::Numpad4 => KeyboardInput::Char('4'),
            ggez::event::KeyCode::Numpad5 => KeyboardInput::Char('5'),
            ggez::event::KeyCode::Numpad6 => KeyboardInput::Char('6'),
            ggez::event::KeyCode::Numpad7 => KeyboardInput::Char('7'),
            ggez::event::KeyCode::Numpad8 => KeyboardInput::Char('8'),
            ggez::event::KeyCode::Numpad9 => KeyboardInput::Char('9'),
            ggez::event::KeyCode::Numpad0 => KeyboardInput::Char('0'),
            ggez::event::KeyCode::F1 => KeyboardInput::Function(1),
            ggez::event::KeyCode::F2 => KeyboardInput::Function(2),
            ggez::event::KeyCode::F3 => KeyboardInput::Function(3),
            ggez::event::KeyCode::F4 => KeyboardInput::Function(4),
            ggez::event::KeyCode::F5 => KeyboardInput::Function(5),
            ggez::event::KeyCode::F6 => KeyboardInput::Function(6),
            ggez::event::KeyCode::F7 => KeyboardInput::Function(7),
            ggez::event::KeyCode::F8 => KeyboardInput::Function(8),
            ggez::event::KeyCode::F9 => KeyboardInput::Function(9),
            ggez::event::KeyCode::F10 => KeyboardInput::Function(10),
            ggez::event::KeyCode::F11 => KeyboardInput::Function(11),
            ggez::event::KeyCode::F12 => KeyboardInput::Function(12),
            ggez::event::KeyCode::F13 => KeyboardInput::Function(13),
            ggez::event::KeyCode::F14 => KeyboardInput::Function(14),
            ggez::event::KeyCode::F15 => KeyboardInput::Function(15),
            ggez::event::KeyCode::F16 => KeyboardInput::Function(16),
            ggez::event::KeyCode::F17 => KeyboardInput::Function(17),
            ggez::event::KeyCode::F18 => KeyboardInput::Function(18),
            ggez::event::KeyCode::F19 => KeyboardInput::Function(19),
            ggez::event::KeyCode::F20 => KeyboardInput::Function(20),
            ggez::event::KeyCode::F21 => KeyboardInput::Function(21),
            ggez::event::KeyCode::F22 => KeyboardInput::Function(22),
            ggez::event::KeyCode::F23 => KeyboardInput::Function(23),
            ggez::event::KeyCode::F24 => KeyboardInput::Function(24),
            ggez::event::KeyCode::At => KeyboardInput::Char('@'),
            ggez::event::KeyCode::Plus => KeyboardInput::Char('+'),
            ggez::event::KeyCode::Minus => KeyboardInput::Char('-'),
            ggez::event::KeyCode::Equals => key_char_shift('=', '+'),
            ggez::event::KeyCode::Backslash => KeyboardInput::Char('\\'),
            ggez::event::KeyCode::Grave => KeyboardInput::Char('`'),
            ggez::event::KeyCode::Apostrophe => KeyboardInput::Char('\''),
            ggez::event::KeyCode::LBracket => KeyboardInput::Char('['),
            ggez::event::KeyCode::RBracket => KeyboardInput::Char(']'),
            ggez::event::KeyCode::Period => KeyboardInput::Char('.'),
            ggez::event::KeyCode::Comma => KeyboardInput::Char(','),
            ggez::event::KeyCode::Slash => KeyboardInput::Char('/'),
            ggez::event::KeyCode::NumpadAdd => KeyboardInput::Char('+'),
            ggez::event::KeyCode::NumpadSubtract => KeyboardInput::Char('-'),
            ggez::event::KeyCode::NumpadMultiply => KeyboardInput::Char('*'),
            ggez::event::KeyCode::NumpadDivide => KeyboardInput::Char('/'),
            ggez::event::KeyCode::PageUp => KeyboardInput::PageUp,
            ggez::event::KeyCode::PageDown => KeyboardInput::PageDown,
            ggez::event::KeyCode::Home => KeyboardInput::Home,
            ggez::event::KeyCode::End => KeyboardInput::End,
            ggez::event::KeyCode::Up => KeyboardInput::Up,
            ggez::event::KeyCode::Down => KeyboardInput::Down,
            ggez::event::KeyCode::Left => KeyboardInput::Left,
            ggez::event::KeyCode::Right => KeyboardInput::Right,
            ggez::event::KeyCode::Return => keys::RETURN,
            ggez::event::KeyCode::Escape => keys::ESCAPE,
            other => {
                log::warn!("Unhandled input: {:?}", other);
                return;
            }
        };
        if let Some(ControlFlow::Exit) = on_input(
            &mut self.chargrid_component,
            Input::Keyboard(input),
            &self.chargrid_frame_buffer,
        ) {
            ggez::event::quit(ctx);
        }
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut ggez::Context,
        button: ggez::event::MouseButton,
        x: f32,
        y: f32,
    ) {
        if let Some(button) = Self::convert_mouse_button(button) {
            self.current_mouse_button = Some(button);
            let coord = self.convert_mouse_position(x, y);
            self.current_mouse_position = coord;
            let input = MouseInput::MousePress { button, coord };
            if let Some(ControlFlow::Exit) = on_input(
                &mut self.chargrid_component,
                Input::Mouse(input),
                &self.chargrid_frame_buffer,
            ) {
                ggez::event::quit(ctx);
            }
        }
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut ggez::Context,
        button: ggez::event::MouseButton,
        x: f32,
        y: f32,
    ) {
        if let Some(button) = Self::convert_mouse_button(button) {
            self.current_mouse_button = None;
            let coord = self.convert_mouse_position(x, y);
            self.current_mouse_position = coord;
            let input = MouseInput::MouseRelease {
                button: Ok(button),
                coord,
            };
            if let Some(ControlFlow::Exit) = on_input(
                &mut self.chargrid_component,
                Input::Mouse(input),
                &self.chargrid_frame_buffer,
            ) {
                ggez::event::quit(ctx);
            }
        }
    }

    fn mouse_motion_event(&mut self, ctx: &mut ggez::Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        let coord = self.convert_mouse_position(x, y);
        self.current_mouse_position = coord;
        let input = MouseInput::MouseMove {
            coord,
            button: self.current_mouse_button,
        };
        if let Some(ControlFlow::Exit) = on_input(
            &mut self.chargrid_component,
            Input::Mouse(input),
            &self.chargrid_frame_buffer,
        ) {
            ggez::event::quit(ctx);
        }
    }

    fn mouse_wheel_event(&mut self, ctx: &mut ggez::Context, x: f32, y: f32) {
        let mut handle = |direction| {
            let coord = self.current_mouse_position;
            let input = MouseInput::MouseScroll { direction, coord };
            if let Some(ControlFlow::Exit) = on_input(
                &mut self.chargrid_component,
                Input::Mouse(input),
                &self.chargrid_frame_buffer,
            ) {
                ggez::event::quit(ctx);
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
    }

    fn quit_event(&mut self, _ctx: &mut ggez::Context) -> bool {
        if let Some(ControlFlow::Exit) = on_input(
            &mut self.chargrid_component,
            Input::Keyboard(keys::ETX),
            &self.chargrid_frame_buffer,
        ) {
            false
        } else {
            true
        }
    }

    #[cfg(feature = "gamepad")]
    fn gamepad_button_down_event(
        &mut self,
        ctx: &mut ggez::Context,
        btn: ggez::event::Button,
        id: ggez::event::GamepadId,
    ) {
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
                return;
            }
        };
        let input = GamepadInput {
            button,
            id: integer_id,
        };
        if let Some(ControlFlow::Exit) = on_input(
            &mut self.chargrid_component,
            Input::Gamepad(input),
            &self.chargrid_frame_buffer,
        ) {
            ggez::event::quit(ctx);
        }
    }
}

pub struct WindowHandle {}

impl WindowHandle {
    pub fn fullscreen(&self) -> bool {
        false
    }
    pub fn set_fullscreen(&self, _fullscreen: bool) {
        log::error!("Setting fullscreen not implemented!");
    }
}

impl Context {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn window_handle(&self) -> WindowHandle {
        WindowHandle {}
    }

    pub fn run_component<C>(self, component: C) -> !
    where
        C: 'static + Component<State = (), Output = Option<ControlFlow>>,
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
            normal: ggez::graphics::Font::new_glyph_font_bytes(&mut ctx, &config.font_bytes.normal)
                .expect("failed to load normal font"),
            bold: ggez::graphics::Font::new_glyph_font_bytes(&mut ctx, &config.font_bytes.bold)
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
        ggez::event::run(
            ctx,
            events_loop,
            GgezApp {
                fonts,
                chargrid_component: component,
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
