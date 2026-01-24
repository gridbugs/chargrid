mod input;
mod text_renderer;
mod wgpu_context;

use std::sync::Arc;
pub use wgpu_context::*;

#[derive(Clone, Debug)]
pub struct FontBytes {
    pub normal: Arc<Vec<u8>>,
    pub bold: Arc<Vec<u8>>,
}

impl FontBytes {
    pub fn new(normal: impl Into<Vec<u8>>, bold: impl Into<Vec<u8>>) -> Self {
        Self {
            normal: Arc::new(normal.into()),
            bold: Arc::new(bold.into()),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Dimensions<T> {
    pub width: T,
    pub height: T,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub title: String,
    pub dimensions_px: Dimensions<f64>,
    pub resizable: bool,
    pub font_bytes: FontBytes,
    pub font_size_px: f32,
    pub cell_dimensions_px: Dimensions<f64>,
    pub underline_width_cell_ratio: f64,
    pub underline_top_offset_cell_ratio: f64,
    pub force_secondary_adapter: bool,
}
