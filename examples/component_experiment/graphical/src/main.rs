use chargrid_component::*;
use chargrid_component_common::{fade, menu, signal, text};
use chargrid_graphical::*;
use std::time::Duration;

#[derive(Clone)]
enum MenuItem {
    String(String),
    Quit,
}

struct HelloWorld {
    title: text::StyledStringWordWrapped,
    menu: menu::PureMenu<MenuItem>,
}

impl HelloWorld {
    fn new() -> Self {
        Self {
            title: text::StyledString {
                string: "Hello, World!".to_string(),
                style: Style {
                    foreground: Some(Rgba32::new_rgb(0, 255, 255)),
                    background: Some(Rgba32::new_rgb(255, 0, 255)),
                    underline: Some(true),
                    bold: Some(true),
                },
            }
            .wrap_word(),
            menu: {
                use menu::builder::*;
                use signal::*;
                let _make_identifier = |s: &str| {
                    identifier::static_(
                        StyledString {
                            string: format!("> {}", s),
                            style: Style {
                                bold: Some(true),
                                foreground: Some(rgba32_grey(255)),
                                ..Style::default()
                            },
                        },
                        StyledString {
                            string: format!("  {}", s),
                            style: Style {
                                bold: Some(false),
                                foreground: Some(rgba32_grey(127)),
                                ..Style::default()
                            },
                        },
                    )
                };
                let make_identifier = |s: &str| {
                    let string = s.to_string();
                    let fade_fg =
                        fade::linear(rgba32_grey(100), rgba32_grey(0), Duration::from_millis(100));
                    let fade_bg = fade::linear(
                        rgba32_grey(255),
                        rgba32_grey(200),
                        Duration::from_millis(200),
                    );
                    let dots = Linear::with_step_duration(Duration::from_millis(50)).min(3);
                    let blink = SquareWave01::with_half_period(Duration::from_millis(250));
                    identifier::dynamic_fn(move |ctx| {
                        if ctx.is_selected {
                            write!(&mut ctx.component.string, "> {}", string).unwrap();
                            for _ in 0..dots.eval(ctx.since_change) {
                                write!(&mut ctx.component.string, ".").unwrap();
                            }
                            if blink.eval_bool(ctx.since_change) {
                                write!(&mut ctx.component.string, "_").unwrap();
                            }
                            ctx.component.style = Style {
                                bold: Some(true),
                                foreground: Some(fade_fg.eval(ctx.since_change)),
                                background: Some(fade_bg.eval(ctx.since_change)),
                                ..Style::default()
                            };
                        } else {
                            write!(&mut ctx.component.string, "  {}", string).unwrap();
                            ctx.component.style = Style {
                                bold: Some(false),
                                foreground: Some(rgba32_grey(127)),
                                background: ctx.style_prev.background.map(|bg| {
                                    fade::linear(bg, rgba32_grey(0), Duration::from_millis(150))
                                        .eval(ctx.since_change)
                                }),
                                ..Style::default()
                            };
                        }
                    })
                };
                menu_builder()
                    .add_item(
                        item(
                            MenuItem::String("foo".to_string()),
                            make_identifier("[F]oo"),
                        )
                        .add_hotkey_char('f'),
                    )
                    .add_item(
                        item(
                            MenuItem::String("bar".to_string()),
                            make_identifier("[B]ar"),
                        )
                        .add_hotkey_char('b'),
                    )
                    .add_item(item(MenuItem::Quit, make_identifier("[Q]uit")).add_hotkey_char('q'))
                    .build()
                    .pure()
            },
        }
    }
}

impl PureComponent for HelloWorld {
    type Output = Option<ControlFlow>;

    fn render(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        self.title.render(
            ctx.add_offset(Coord::new(2, 3))
                .constrain_size_by(Coord::new(8, 5)),
            fb,
        );
        self.menu.render(ctx.add_offset(Coord::new(4, 6)), fb);
    }

    fn update(&mut self, ctx: Ctx, event: Event) -> Self::Output {
        match event {
            Event::Input(input::Input::Keyboard(input::keys::ESCAPE)) => Some(ControlFlow::Exit),
            _ => {
                if let Some(item) = self.menu.update(ctx.add_offset(Coord::new(4, 6)), event) {
                    match item {
                        MenuItem::String(s) => {
                            self.title.styled_string.string = s;
                            None
                        }
                        MenuItem::Quit => Some(ControlFlow::Exit),
                    }
                } else {
                    None
                }
            }
        }
    }
}

fn main() {
    let context = Context::new(Config {
        font_bytes: FontBytes {
            normal: include_bytes!("./fonts/PxPlus_IBM_CGAthin.ttf").to_vec(),
            bold: include_bytes!("./fonts/PxPlus_IBM_CGA.ttf").to_vec(),
        },
        title: "Tetris".to_string(),
        window_dimensions_px: Dimensions {
            width: 640.,
            height: 480.,
        },
        cell_dimensions_px: Dimensions {
            width: 32.,
            height: 32.,
        },
        font_scale: Dimensions {
            width: 32.,
            height: 32.,
        },
        underline_width_cell_ratio: 0.1,
        underline_top_offset_cell_ratio: 0.8,
        resizable: false,
    });
    let app_wrapper = AppWrapper {
        component: HelloWorld::new().component(),
        frame_buffer: FrameBuffer::new(context.grid_size()),
    };
    context.run_app(app_wrapper);
}
