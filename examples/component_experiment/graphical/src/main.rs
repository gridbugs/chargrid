use chargrid_component::*;
use chargrid_component_common::{menu, text};
use chargrid_graphical::*;

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
                let make_identifier = |s: &str| {
                    identifiers_styled_strings(
                        format!("> {}", s),
                        format!("  {}", s),
                        Style {
                            bold: Some(true),
                            foreground: Some(rgba32_grey(255)),
                            ..Style::default()
                        },
                        Style {
                            bold: Some(false),
                            foreground: Some(rgba32_grey(127)),
                            ..Style::default()
                        },
                    )
                };
                menu_builder()
                    .add_choice(
                        choice(
                            MenuItem::String("foo".to_string()),
                            make_identifier("[F]oo"),
                        )
                        .add_hotkey(KeyboardInput::Char('f')),
                    )
                    .add_choice(
                        choice(
                            MenuItem::String("bar".to_string()),
                            make_identifier("[B]ar"),
                        )
                        .add_hotkey(KeyboardInput::Char('b')),
                    )
                    .add_choice(
                        choice(MenuItem::Quit, make_identifier("[Q]uit"))
                            .add_hotkey(KeyboardInput::Char('q')),
                    )
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
                if let Some(choice) = self.menu.update(ctx.add_offset(Coord::new(4, 6)), event) {
                    match choice {
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
