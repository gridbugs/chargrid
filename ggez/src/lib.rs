use chargrid_app::{App, ControlFlow};
pub use chargrid_graphical_common::*;
use chargrid_input::{keys, Input, KeyboardInput};
use chargrid_render::{Buffer, Rgb24, Size, ViewContext};
use std::time::Instant;

pub struct Context {
    config: Config,
}

struct Fonts {
    normal: ggez::graphics::Font,
    bold: ggez::graphics::Font,
}

struct GgezApp<A: App + 'static> {
    fonts: Fonts,
    chargrid_app: A,
    buffer: Buffer,
    last_frame: Instant,
    font_scale: ggez::graphics::PxScale,
    underline_mesh: ggez::graphics::Mesh,
    background_mesh: ggez::graphics::Mesh,
    cell_width: f32,
    cell_height: f32,
}

impl<A: App + 'static> ggez::event::EventHandler for GgezApp<A> {
    fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        const DESIRED_FPS: u32 = 60;
        while ggez::timer::check_update_time(ctx, DESIRED_FPS) {}
        let now = Instant::now();
        self.buffer.clear();
        let view_context = ViewContext::default_with_size(self.buffer.size());
        if let Some(ControlFlow::Exit) =
            self.chargrid_app
                .on_frame(now - self.last_frame, view_context, &mut self.buffer)
        {
            ggez::event::quit(ctx);
        }
        self.last_frame = now;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        ggez::graphics::clear(ctx, [0., 0., 0., 1.0].into());
        for (coord, cell) in self.buffer.enumerate() {
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
                    Some(cell.foreground_colour.to_f32_rgba(1.).into()),
                );
            }
            if cell.background_colour != Rgb24::new(0, 0, 0) {
                ggez::graphics::draw(
                    ctx,
                    &self.background_mesh,
                    ggez::graphics::DrawParam::default()
                        .dest(ggez::mint::Point2 {
                            x: coord.x as f32 * self.cell_width,
                            y: coord.y as f32 * self.cell_height,
                        })
                        .color(cell.background_colour.to_f32_rgba(1.).into()),
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
                        .color(cell.foreground_colour.to_f32_rgba(1.).into()),
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
        _keymods: ggez::input::keyboard::KeyMods,
        _repeat: bool,
    ) {
        let input = match keycode {
            ggez::event::KeyCode::Up => Input::Keyboard(KeyboardInput::Up),
            ggez::event::KeyCode::Down => Input::Keyboard(KeyboardInput::Down),
            ggez::event::KeyCode::Left => Input::Keyboard(KeyboardInput::Left),
            ggez::event::KeyCode::Right => Input::Keyboard(KeyboardInput::Right),
            ggez::event::KeyCode::Return => Input::Keyboard(keys::RETURN),
            ggez::event::KeyCode::Escape => Input::Keyboard(keys::ESCAPE),
            _ => return,
        };
        if let Some(ControlFlow::Exit) = self.chargrid_app.on_input(input) {
            ggez::event::quit(ctx);
        }
    }

    fn quit_event(&mut self, _ctx: &mut ggez::Context) -> bool {
        if let Some(ControlFlow::Exit) = self.chargrid_app.on_input(Input::Keyboard(keys::ETX)) {
            false
        } else {
            true
        }
    }
}

impl Context {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn run_app<A>(self, app: A) -> !
    where
        A: App + 'static,
    {
        let Self { config } = self;
        let grid_size = Size::new(
            (config.window_dimensions_px.width as f64 / config.cell_dimensions_px.width) as u32,
            (config.window_dimensions_px.height as f64 / config.cell_dimensions_px.height) as u32,
        );
        let buffer = Buffer::new(grid_size);
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
                chargrid_app: app,
                buffer,
                last_frame: Instant::now(),
                font_scale: ggez::graphics::PxScale {
                    x: config.font_scale.width as f32,
                    y: config.font_scale.height as f32,
                },
                underline_mesh,
                background_mesh,
                cell_width: config.cell_dimensions_px.width as f32,
                cell_height: config.cell_dimensions_px.height as f32,
            },
        )
    }
}
