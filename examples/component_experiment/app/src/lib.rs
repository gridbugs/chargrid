use chargrid::{border, core::*, fade, menu, pad_to, signal, text};
use std::time::Duration;

#[derive(Clone)]
enum MenuItem {
    String(String),
    Quit,
}

pub struct HelloWorld {
    title: text::StyledStringWordWrapped,
    menu: border::Border<pad_to::PadTo<menu::Menu<MenuItem>>>,
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
                let make_identifier_static = |s: &str| {
                    identifier::static_(
                        vec![
                            StyledString {
                                string: ">".to_string(),
                                style: Style {
                                    bold: Some(true),
                                    foreground: Some(Rgba32::new_rgb(255, 0, 0)),
                                    ..Style::default()
                                },
                            },
                            StyledString {
                                string: format!(" {}", s),
                                style: Style {
                                    bold: Some(true),
                                    foreground: Some(Rgba32::new_grey(255)),
                                    ..Style::default()
                                },
                            },
                        ],
                        StyledString {
                            string: format!("  {}", s),
                            style: Style {
                                bold: Some(false),
                                foreground: Some(Rgba32::new_grey(127)),
                                ..Style::default()
                            },
                        },
                    )
                };
                let make_identifier = |s: &str, c: Rgba32| {
                    let string = s.to_string();
                    let fade_fg =
                        fade::linear(Rgba32::new_grey(100), c, Duration::from_millis(100));
                    let fade_bg = fade::linear(
                        Rgba32::new_grey(255),
                        Rgba32::new_grey(200),
                        Duration::from_millis(200),
                    );
                    let dots = Linear::with_step_duration(Duration::from_millis(50)).min(3);
                    let blink = SmoothSquareWave::new(
                        Duration::from_millis(200),
                        Duration::from_millis(100),
                    );
                    let rainbow = vec![
                        Rgba32::new_rgb(255, 0, 0),
                        Rgba32::new_rgb(255, 255, 0),
                        Rgba32::new_rgb(0, 255, 0),
                        Rgba32::new_rgb(0, 255, 255),
                        Rgba32::new_rgb(0, 0, 255),
                        Rgba32::new_rgb(255, 0, 255),
                    ];
                    let mut count = 0;
                    identifier::dynamic_fn(4, move |ctx| {
                        if ctx.is_selected {
                            write!(&mut ctx.component.parts[0].string, "> ").unwrap();
                            ctx.component.parts[0].style = Style {
                                bold: Some(true),
                                foreground: Some(rainbow[(count / 10) % rainbow.len()]),
                                ..Style::default()
                            };
                            count += 1;
                            write!(&mut ctx.component.parts[1].string, "{}", string).unwrap();
                            ctx.component.parts[1].style = Style {
                                bold: Some(true),
                                foreground: Some(fade_fg.eval(ctx.since_change)),
                                background: Some(fade_bg.eval(ctx.since_change)),
                                ..Style::default()
                            };
                            for _ in 0..dots.eval(ctx.since_change) {
                                write!(&mut ctx.component.parts[2].string, ".").unwrap();
                            }
                            write!(&mut ctx.component.parts[3].string, " ").unwrap();
                            ctx.component.parts[3].style = Style {
                                bold: Some(false),
                                background: Some(
                                    Rgba32::new_grey(255)
                                        .with_a(255 - blink.eval(ctx.since_change)),
                                ),
                                ..Style::default()
                            };
                        } else {
                            write!(&mut ctx.component.parts[0].string, "  ").unwrap();
                            write!(&mut ctx.component.parts[1].string, "{}", string).unwrap();
                            ctx.component.parts[1].style = Style {
                                bold: Some(false),
                                foreground: Some(Rgba32::new_grey(127)),
                                background: ctx.styles_prev[1].background.map(|bg| {
                                    fade::linear(
                                        bg,
                                        Rgba32::new(0, 0, 0, 0),
                                        Duration::from_millis(150),
                                    )
                                    .eval(ctx.since_change)
                                }),
                                ..Style::default()
                            };
                        }
                    })
                };
                let menu = menu_builder()
                    .add_item(
                        item(
                            MenuItem::String("foo".to_string()),
                            make_identifier("[F]oo", Rgba32::new_rgb(255, 0, 0)),
                        )
                        .add_hotkey_char('f'),
                    )
                    .add_item(
                        item(
                            MenuItem::String("bar".to_string()),
                            make_identifier("[B]ar", Rgba32::new_rgb(0, 0, 255)),
                        )
                        .add_hotkey_char('b'),
                    )
                    .add_space()
                    .add_item(
                        item(MenuItem::Quit, make_identifier_static("[Q]uit")).add_hotkey_char('q'),
                    )
                    .build();
                border::Border {
                    component: pad_to::PadTo {
                        component: menu,
                        size: Size::new(16, 0),
                    },
                    style: border::BorderStyle {
                        title: Some("omg a border".to_string()),
                        padding: border::BorderPadding {
                            top: 1,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                }
            },
        }
    }
}

fn title_ctx<'a>(ctx: Ctx<'a>) -> Ctx<'a> {
    ctx.add_offset(Coord::new(2, 1)).set_size(Size::new(8, 2))
}

fn menu_ctx<'a>(ctx: Ctx<'a>) -> Ctx<'a> {
    ctx.add_offset(Coord::new(1, 4))
}

impl Component for HelloWorld {
    type Output = app::Output;
    type State = ();

    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        fb.clear_with_background(Rgba32::new_rgb(0, 0, 100));
        self.title.render(state, title_ctx(ctx), fb);
        self.menu.render(state, menu_ctx(ctx), fb);
    }

    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        match event {
            Event::Input(input::Input::Keyboard(input::keys::ESCAPE)) => Some(app::Exit),
            _ => {
                if let Some(item) = self.menu.update(state, menu_ctx(ctx), event) {
                    match item {
                        MenuItem::String(s) => {
                            self.title.styled_string.string = s;
                            None
                        }
                        MenuItem::Quit => Some(app::Exit),
                    }
                } else {
                    None
                }
            }
        }
    }

    fn size(&self, _state: &Self::State, ctx: Ctx) -> Size {
        ctx.bounding_box.size()
    }
}

pub fn app() -> impl Component<State = (), Output = app::Output> {
    HelloWorld::new()
}
