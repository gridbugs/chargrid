use chargrid_wgpu::*;
use colour_grid_app::app;

fn main() {
    let (window, event_loop) = make_window_and_event_loop(
        "Colour Grid",
        Dimensions {
            width: 640.,
            height: 480.,
        },
        false,
    );
    let context = Context::new(
        &window,
        event_loop,
        Config {
            font_bytes: FontBytes {
                normal: include_bytes!("./fonts/PxPlus_IBM_CGAthin.ttf").to_vec(),
                bold: include_bytes!("./fonts/PxPlus_IBM_CGA.ttf").to_vec(),
            },
            cell_dimensions_px: Dimensions {
                width: 14.,
                height: 14.,
            },
            font_scale: Dimensions {
                width: 14.,
                height: 14.,
            },
            underline_width_cell_ratio: 0.1,
            underline_top_offset_cell_ratio: 0.8,
            force_secondary_adapter: false,
        },
    );
    context.run(app());
}
