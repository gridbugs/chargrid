use chargrid_sdl2::*;
use component_experiment_app::app;

fn main() {
    let context = Context::new(Config {
        font_bytes: FontBytes {
            normal: include_bytes!("./fonts/PxPlus_IBM_CGAthin.ttf").to_vec(),
            bold: include_bytes!("./fonts/PxPlus_IBM_CGA.ttf").to_vec(),
        },
        title: "Component Experiment".to_string(),
        window_dimensions_px: Dimensions {
            width: 320.,
            height: 240.,
        },
        cell_dimensions_px: Dimensions {
            width: 12.,
            height: 12.,
        },
        font_point_size: 12,
        character_cell_offset: Dimensions {
            width: 0.,
            height: -1.,
        },
        underline_width_cell_ratio: 0.1,
        underline_top_offset_cell_ratio: 0.8,
        resizable: false,
    });
    context.run(app());
}
