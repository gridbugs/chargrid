#[cfg(feature = "gamepad")]
use chargrid_gamepad::GamepadContext;
use chargrid_input::{keys, Input, MouseButton, MouseInput, ScrollDirection};
use chargrid_runtime::{app, on_frame, on_input, Component, Coord, FrameBuffer, Rgba32, Size};
use sdl2::{event::Event, pixels::Color, rect::Rect, rwops::RWops, surface::Surface, ttf};
use std::{
    thread,
    time::{Duration, Instant},
};
mod input;

const FRAME_DURATION: Duration = Duration::from_micros(1_000_000 / 60);

pub struct FontBytes {
    pub normal: Vec<u8>,
    pub bold: Vec<u8>,
}

struct FontGeneric<T> {
    normal: T,
    bold: T,
}

type FontSdl2Ttf<'ttf_context, 'font_data> = FontGeneric<ttf::Font<'ttf_context, 'font_data>>;

impl FontBytes {
    fn load_single<'ttf_context, 'font_data>(
        font_data: &'font_data [u8],
        ttf_context: &'ttf_context ttf::Sdl2TtfContext,
        pt_size: u16,
    ) -> ttf::Font<'ttf_context, 'font_data> {
        let rwops = RWops::from_bytes(font_data).expect("failed to create rwops for front data");
        ttf_context
            .load_font_from_rwops(rwops, pt_size)
            .expect("failed to load font data")
    }

    fn load<'ttf_context, 'font_data>(
        &'font_data self,
        ttf_context: &'ttf_context ttf::Sdl2TtfContext,
        pt_size: u16,
    ) -> FontSdl2Ttf<'ttf_context, 'font_data> {
        FontGeneric {
            normal: Self::load_single(&self.normal, ttf_context, pt_size),
            bold: Self::load_single(&self.bold, ttf_context, pt_size),
        }
    }
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
    pub font_point_size: u16,
    pub underline_width_cell_ratio: f64,
    pub underline_top_offset_cell_ratio: f64,
    pub resizable: bool,
}

pub struct Context {
    config: Config,
}

impl Context {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn run<C>(self, mut component: C)
    where
        C: 'static + Component<State = (), Output = app::Output>,
    {
        let Self { config } = self;
        let sdl_context = sdl2::init().expect("failed to initialize sdl2");
        let video_subsys = sdl_context
            .video()
            .expect("failed to connect to video subsystem");
        let ttf_context = ttf::init().expect("failed to initialize ttf context");
        let font = config.font_bytes.load(&ttf_context, config.font_point_size);
        let mut window_builder = video_subsys.window(
            config.title.as_str(),
            config.window_dimensions_px.width as u32,
            config.window_dimensions_px.height as u32,
        );
        window_builder.position_centered().opengl();
        if config.resizable {
            window_builder.resizable();
        }
        let window = window_builder.build().expect("failed to open window");
        let mut canvas = window
            .into_canvas()
            .accelerated()
            .build()
            .expect("failed to create canvas");
        #[cfg(feature = "gamepad")]
        let mut gamepad_context = GamepadContext::new();
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
        canvas.clear();
        let texture_creator = canvas.texture_creator();
        let default_pixel_format = texture_creator.default_pixel_format();
        let mut text_surface = Surface::new(
            config.window_dimensions_px.width as u32,
            config.window_dimensions_px.height as u32,
            default_pixel_format,
        )
        .expect("failed to create surface");
        let grid_size = Size::new(
            (config.window_dimensions_px.width as f64 / config.cell_dimensions_px.width) as u32,
            (config.window_dimensions_px.height as f64 / config.cell_dimensions_px.height) as u32,
        );
        let px_to_coord = |x: i32, y: i32| Coord {
            x: (x as f64 / config.cell_dimensions_px.width) as i32,
            y: (y as f64 / config.cell_dimensions_px.height) as i32,
        };
        let mut current_mouse_position = Coord::new(0, 0);
        let mut chargrid_frame_buffer = FrameBuffer::new(grid_size);
        'mainloop: loop {
            let frame_start = Instant::now();
            #[cfg(feature = "gamepad")]
            for input in gamepad_context.drain_input() {
                if let Some(app::Exit) = on_input(
                    &mut component,
                    chargrid_input::Input::Gamepad(input),
                    &chargrid_frame_buffer,
                ) {
                    break 'mainloop;
                }
            }
            for event in sdl_context
                .event_pump()
                .expect("failed to create event pump")
                .poll_iter()
            {
                let input = match event {
                    Event::KeyDown {
                        keycode: Some(keycode),
                        keymod,
                        ..
                    } => input::sdl2_to_chargrid(keycode, keymod).map(Input::Keyboard),
                    Event::MouseMotion {
                        mousestate, x, y, ..
                    } => {
                        // chargrid doesn't handle dragging the mouse with multiple buttons pressed
                        let button = if mousestate.left() {
                            Some(MouseButton::Left)
                        } else if mousestate.right() {
                            Some(MouseButton::Right)
                        } else if mousestate.middle() {
                            Some(MouseButton::Middle)
                        } else {
                            None
                        };
                        current_mouse_position = px_to_coord(x, y);
                        Some(Input::Mouse(MouseInput::MouseMove {
                            button,
                            coord: current_mouse_position,
                        }))
                    }
                    Event::MouseButtonDown {
                        mouse_btn, x, y, ..
                    } => {
                        let button = match mouse_btn {
                            sdl2::mouse::MouseButton::Left => Some(MouseButton::Left),
                            sdl2::mouse::MouseButton::Right => Some(MouseButton::Right),
                            sdl2::mouse::MouseButton::Middle => Some(MouseButton::Middle),
                            _ => None,
                        };
                        button.map(|button| {
                            current_mouse_position = px_to_coord(x, y);
                            Input::Mouse(MouseInput::MousePress {
                                button,
                                coord: current_mouse_position,
                            })
                        })
                    }
                    Event::MouseButtonUp {
                        mouse_btn, x, y, ..
                    } => {
                        let button = match mouse_btn {
                            sdl2::mouse::MouseButton::Left => Some(MouseButton::Left),
                            sdl2::mouse::MouseButton::Right => Some(MouseButton::Right),
                            sdl2::mouse::MouseButton::Middle => Some(MouseButton::Middle),
                            _ => None,
                        };
                        button.map(|button| {
                            current_mouse_position = px_to_coord(x, y);
                            Input::Mouse(MouseInput::MouseRelease {
                                button: Ok(button),
                                coord: current_mouse_position,
                            })
                        })
                    }
                    Event::MouseWheel { x, y, .. } => {
                        let direction = if y > 0 {
                            Some(ScrollDirection::Up)
                        } else if y < 0 {
                            Some(ScrollDirection::Down)
                        } else if x > 0 {
                            Some(ScrollDirection::Left)
                        } else if x < 0 {
                            Some(ScrollDirection::Right)
                        } else {
                            None
                        };
                        direction.map(|direction| {
                            Input::Mouse(MouseInput::MouseScroll {
                                direction,
                                coord: current_mouse_position,
                            })
                        })
                    }
                    Event::Quit { .. } => Some(Input::Keyboard(keys::ETX)),
                    _ => None,
                };
                if let Some(input) = input {
                    if let Some(app::Exit) = on_input(&mut component, input, &chargrid_frame_buffer)
                    {
                        break 'mainloop;
                    }
                }
            }
            if let Some(app::Exit) =
                on_frame(&mut component, FRAME_DURATION, &mut chargrid_frame_buffer)
            {
                break;
            }
            canvas.clear();
            text_surface
                .fill_rect(
                    None,
                    Color {
                        r: 0,
                        g: 0,
                        b: 0,
                        a: 0,
                    },
                )
                .expect("failed to clear surface");
            for (coord, cell) in chargrid_frame_buffer.enumerate() {
                let dst = Rect::new(
                    (coord.x as f64 * config.cell_dimensions_px.width) as i32,
                    (coord.y as f64 * config.cell_dimensions_px.height) as i32,
                    config.cell_dimensions_px.width as u32,
                    config.cell_dimensions_px.height as u32,
                );
                let bg_colour = {
                    let Rgba32 { r, g, b, a } = cell.background;
                    Color::RGBA(r, g, b, a)
                };
                let fg_colour = {
                    let Rgba32 { r, g, b, a } = cell.foreground;
                    Color::RGBA(r, g, b, a)
                };
                text_surface
                    .fill_rect(dst, bg_colour)
                    .expect("failed to fill background");
                if cell.underline {
                    let rect = Rect::new(
                        dst.left(),
                        dst.top()
                            + (config.underline_top_offset_cell_ratio
                                * config.cell_dimensions_px.height)
                                as i32,
                        config.cell_dimensions_px.width as u32,
                        (config.underline_width_cell_ratio * config.cell_dimensions_px.height)
                            as u32,
                    );
                    text_surface
                        .fill_rect(rect, fg_colour)
                        .expect("failed to fill underline");
                }
                if cell.character == ' ' {
                    continue;
                }
                let font = if cell.bold { &font.bold } else { &font.normal };
                let surface = font
                    .render_char(cell.character)
                    .solid(fg_colour)
                    .expect("failed to render character");
                surface
                    .blit(None, &mut text_surface, Some(dst))
                    .expect("failed to copy character to surface");
            }
            let text_texture = text_surface
                .as_texture(&texture_creator)
                .expect("failed to create texture from surface");
            canvas
                .copy(&text_texture, None, None)
                .expect("failed to copy rendered character to canvas");
            canvas.present();
            let since_frame_start = frame_start.elapsed();
            if let Some(until_next_frame) = FRAME_DURATION.checked_sub(since_frame_start) {
                thread::sleep(until_next_frame);
            }
        }
    }
}
