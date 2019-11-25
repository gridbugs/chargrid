use colour_picker_prototty::app;
use prototty_graphical_::*;

fn main() {
    let context = Context::new(ContextDescription {
        font_bytes: FontBytes {
            normal: include_bytes!("./fonts/PxPlus_IBM_CGAthin.ttf").to_vec(),
            bold: include_bytes!("./fonts/PxPlus_IBM_CGA.ttf").to_vec(),
        },
        title: "Template Roguelike".to_string(),
        window_dimensions: WindowDimensions::Windowed(Dimensions {
            width: 640.,
            height: 480.,
        }),
        cell_dimensions: Dimensions {
            width: 14.,
            height: 14.,
        },
        font_dimensions: Dimensions {
            width: 14.,
            height: 14.,
        },
        underline_width: 0.1,
        underline_top_offset: 0.8,
    })
    .unwrap();
    context.run_app(app());
}
