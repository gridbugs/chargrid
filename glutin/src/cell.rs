use prototty_grid::*;
use prototty_render::Rgb24;

#[derive(Debug, Clone)]
pub struct Colour(pub [f32; 4]);

impl From<Rgb24> for Colour {
    fn from(rgb24: Rgb24) -> Self {
        Colour([
            rgb24.red as f32 / 255.0,
            rgb24.green as f32 / 255.0,
            rgb24.blue as f32 / 255.0,
            1.0,
        ])
    }
}

impl DefaultForeground for Colour {
    fn default_foreground() -> Self {
        Colour([1.0, 1.0, 1.0, 1.0])
    }
}

impl DefaultBackground for Colour {
    fn default_background() -> Self {
        Colour([0.0, 0.0, 0.0, 1.0])
    }
}
