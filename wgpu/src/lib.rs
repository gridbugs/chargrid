mod input;
mod wgpu_context;

pub use wgpu_context::*;

pub struct FontBytes {
    pub normal: Vec<u8>,
    pub bold: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
pub struct Dimensions<T> {
    pub width: T,
    pub height: T,
}

pub struct Config {
    pub title: String,
    pub font_bytes: FontBytes,
    pub window_dimensions_px: Dimensions<f64>,
    pub cell_dimensions_px: Dimensions<f64>,
    pub font_scale: Dimensions<f64>,
    pub underline_width_cell_ratio: f64,
    pub underline_top_offset_cell_ratio: f64,
    pub resizable: bool,
    pub force_secondary_adapter: bool,
}
