use chargrid_wgpu::*;
use component_experiment_app::app;

fn main() {
    env_logger::init();
    run(
        app(),
        Config {
            title: "Tetris".to_string(),
            dimensions_px: Dimensions {
                width: 640.,
                height: 480.,
            },
            resizable: false,
            font_bytes: FontBytes::new(
                include_bytes!("./fonts/PxPlus_IBM_CGAthin.ttf").to_vec(),
                include_bytes!("./fonts/PxPlus_IBM_CGA.ttf").to_vec(),
            ),
            font_size_px: 32.,
            cell_dimensions_px: Dimensions {
                width: 32.,
                height: 32.,
            },
            character_cell_offset_px: Dimensions {
                width: 0.,
                height: 0.,
            },
            underline_width_cell_ratio: 0.1,
            underline_top_offset_cell_ratio: 0.8,
            force_secondary_adapter: false,
        },
    )
    .unwrap();
}
