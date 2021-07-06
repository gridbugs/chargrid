use chargrid_component as cc;
use chargrid_graphical::*;

struct HelloWorld;
impl cc::PureComponent for HelloWorld {
    type PureOutput = Option<cc::ControlFlow>;

    fn pure_render(&self, ctx: cc::Ctx, fb: &mut cc::FrameBuffer) {
        let s = "Hello, World!";
        let offset = ctx.offset + cc::Coord::new(2, 2);
        for (i, character) in s.chars().enumerate() {
            fb.set_cell(
                offset + cc::Coord::new(i as i32, 0),
                1,
                cc::RenderCell {
                    character: Some(character),
                    style: cc::Style {
                        foreground: Some(cc::Rgba32::new_rgb(0, 255, 255)),
                        background: Some(cc::Rgba32::new_rgb(255, 0, 255)),
                        underline: Some(true),
                        bold: Some(true),
                    },
                },
            );
        }
    }
    fn pure_update(&mut self, _ctx: cc::Ctx, event: cc::Event) -> Self::PureOutput {
        match event {
            cc::Event::Input(cc::Input::Keyboard(cc::input::keys::ESCAPE)) => {
                Some(cc::ControlFlow::Exit)
            }
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
    let app_wrapper = cc::AppWrapper {
        component: HelloWorld,
        frame_buffer: cc::FrameBuffer::new(cc::Size::new(100, 100)),
    };
    context.run_app(app_wrapper);
}
