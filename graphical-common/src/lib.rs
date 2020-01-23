pub struct FontBytes {
    pub normal: Vec<u8>,
    pub bold: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
pub struct Dimensions<T> {
    pub width: T,
    pub height: T,
}

pub type NumPixels = f64;
pub type CellRatio = f64;
pub type FontSourceScale = f32;

pub enum WindowDimensions {
    Windowed(Dimensions<NumPixels>),
    Fullscreen,
}

pub struct ContextDescriptor {
    pub font_bytes: FontBytes,
    pub title: String,
    pub window_dimensions: WindowDimensions,
    pub cell_dimensions: Dimensions<NumPixels>,
    pub font_dimensions: Dimensions<NumPixels>,
    pub font_source_dimensions: Dimensions<FontSourceScale>,
    pub underline_width: CellRatio,
    pub underline_top_offset: CellRatio,
}
