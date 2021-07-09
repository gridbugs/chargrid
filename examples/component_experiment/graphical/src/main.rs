use chargrid_component::*;
use chargrid_component_common::{menu, text};
use chargrid_graphical::*;

struct HelloWorld {
    title: text::StyledStringWordWrapped,
    menu: menu::Menu<String>,
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
            menu: menu::MenuBuilder::default().add_item().build(),
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
    }

    fn update(&mut self, _ctx: Ctx, event: Event) -> Self::Output {
        match event {
            Event::Input(input::Input::Keyboard(input::keys::ESCAPE)) => Some(ControlFlow::Exit),
            _ => None,
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
