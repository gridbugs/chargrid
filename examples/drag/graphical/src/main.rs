use chargrid_graphical::*;
use drag_app::App;

fn main() {
    let context = Context::new(ContextDescriptor {
        font_bytes: FontBytes {
            normal: include_bytes!("./fonts/PxPlus_IBM_CGAthin.ttf").to_vec(),
            bold: include_bytes!("./fonts/PxPlus_IBM_CGA.ttf").to_vec(),
        },
        title: "Drag".to_string(),
        window_dimensions: Dimensions {
            width: 640.,
            height: 480.,
        },
        cell_dimensions: Dimensions { width: 8., height: 8. },
        font_dimensions: Dimensions { width: 8., height: 8. },
        font_source_dimensions: Dimensions { width: 8., height: 8. },
        underline_width: 0.1,
        underline_top_offset: 0.8,
        resizable: false,
    })
    .unwrap();
    context.run_app(App::default());
}
