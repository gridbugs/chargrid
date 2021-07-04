use chargrid_graphical::*;
use pager_app::App;
use std::io::{self, Read};

fn main() -> io::Result<()> {
    env_logger::init();
    let mut text = String::new();
    io::stdin().read_to_string(&mut text)?;
    let context = Context::new(Config {
        font_bytes: FontBytes {
            normal: include_bytes!("./fonts/PxPlus_IBM_CGAthin.ttf").to_vec(),
            bold: include_bytes!("./fonts/PxPlus_IBM_CGA.ttf").to_vec(),
        },
        title: "Pager".to_string(),
        window_dimensions_px: Dimensions {
            width: 640.,
            height: 480.,
        },
        cell_dimensions_px: Dimensions {
            width: 12.,
            height: 12.,
        },
        font_scale: Dimensions {
            width: 12.,
            height: 12.,
        },
        underline_width_cell_ratio: 0.1,
        underline_top_offset_cell_ratio: 0.8,
        resizable: false,
    });
    context.run_app(App::new(text))
}
