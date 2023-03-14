use chargrid_runtime::{app, on_frame, Component, FrameBuffer, Rgba32, Size};
use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Rect, rwops::RWops, ttf};
use std::{
    thread,
    time::{Duration, Instant},
};

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
        let ttf_context = sdl2::ttf::init().expect("failed to initialize ttf context");
        let font = config.font_bytes.load(&ttf_context, config.font_point_size);
        let window = video_subsys
            .window(
                config.title.as_str(),
                config.window_dimensions_px.width as u32,
                config.window_dimensions_px.height as u32,
            )
            .position_centered()
            .opengl()
            .build()
            .expect("failed to open window");
        let mut canvas = window
            .into_canvas()
            .build()
            .expect("failed to create canvas");
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
        canvas.clear();
        let texture_creator = canvas.texture_creator();
        let grid_size = Size::new(
            (config.window_dimensions_px.width as f64 / config.cell_dimensions_px.width) as u32,
            (config.window_dimensions_px.height as f64 / config.cell_dimensions_px.height) as u32,
        );
        let mut chargrid_frame_buffer = FrameBuffer::new(grid_size);
        'mainloop: loop {
            let frame_start = Instant::now();
            for event in sdl_context
                .event_pump()
                .expect("failed to create event pump")
                .poll_iter()
            {
                match event {
                    Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    }
                    | Event::Quit { .. } => break 'mainloop,
                    _ => {}
                }
            }
            if let Some(app::Exit) =
                on_frame(&mut component, FRAME_DURATION, &mut chargrid_frame_buffer)
            {
                break;
            }
            canvas.clear();
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
                canvas.set_draw_color(bg_colour);
                canvas
                    .fill_rect(dst)
                    .expect("failed to fill cell background");
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
                    canvas.set_draw_color(fg_colour);
                    canvas.fill_rect(rect).expect("failed to fill underline");
                }
                if cell.character == ' ' {
                    continue;
                }
                let font = if cell.bold { &font.bold } else { &font.normal };
                let surface = font
                    .render_char(cell.character)
                    .blended(fg_colour)
                    .expect("failed to render character");
                let texture = texture_creator
                    .create_texture_from_surface(&surface)
                    .expect("failed to create texture from surface");
                canvas
                    .copy(&texture, None, Some(dst))
                    .expect("failed to copy rendered character to canvas");
            }
            canvas.present();
            let since_frame_start = frame_start.elapsed();
            if let Some(until_next_frame) = FRAME_DURATION.checked_sub(since_frame_start) {
                thread::sleep(until_next_frame);
            }
        }
    }
}