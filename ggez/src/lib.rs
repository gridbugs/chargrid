use chargrid_app::{App, ControlFlow};
pub use chargrid_graphical_common::*;
use chargrid_input::{keys, Input, KeyboardInput};
use chargrid_render::{Buffer, Size, ViewContext};
use std::time::Instant;

pub struct Context {
    config: Config,
}

struct GgezApp<A: App + 'static> {
    config: Config,
    chargrid_app: A,
    buffer: Buffer,
    last_frame: Instant,
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
            ggez::graphics::queue_text(
                ctx,
                &ggez::graphics::Text::new(cell.character),
                ggez::mint::Point2 {
                    x: coord.x as f32 * self.config.cell_dimensions_px.width as f32,
                    y: coord.y as f32 * self.config.cell_dimensions_px.height as f32,
                },
                Some(cell.foreground_colour.to_f32_rgba(1.).into()),
            );
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
        let (ctx, events_loop) = ggez::ContextBuilder::new(config.title.as_str(), "chargrid_ggez")
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
        ggez::event::run(
            ctx,
            events_loop,
            GgezApp {
                config,
                chargrid_app: app,
                buffer,
                last_frame: Instant::now(),
            },
        )
    }
}
