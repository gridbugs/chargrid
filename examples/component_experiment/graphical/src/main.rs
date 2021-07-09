use chargrid_component::*;
use chargrid_component_common::menu;
use chargrid_graphical::*;

struct HelloWorld;
impl PureComponent for HelloWorld {
    type PureOutput = Option<ControlFlow>;

    fn pure_render(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        let s = "Hello, World!";
        let offset = ctx.bounding_box.coord + Coord::new(2, 2);
        for (i, character) in s.chars().enumerate() {
            fb.set_cell(
                offset + Coord::new(i as i32, 0),
                1,
                RenderCell {
                    character: Some(character),
                    style: Style {
                        foreground: Some(Rgba32::new_rgb(0, 255, 255)),
                        background: Some(Rgba32::new_rgb(255, 0, 255)),
                        underline: Some(true),
                        bold: Some(true),
                    },
                },
            );
        }
    }
    fn pure_update(&mut self, _ctx: Ctx, event: Event) -> Self::PureOutput {
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
        component: HelloWorld,
        frame_buffer: FrameBuffer::new(Size::new(100, 100)),
    };
    context.run_app(app_wrapper);
}
