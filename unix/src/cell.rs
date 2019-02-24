use ansi_colour;
use defaults::*;
use prototty_grid::*;
use prototty_render::Rgb24;

#[derive(Debug, Clone)]
pub struct Colour(pub ansi_colour::Colour);

impl From<Rgb24> for Colour {
    fn from(rgb24: Rgb24) -> Self {
        Colour(ansi_colour::Colour::from_rgb24(rgb24))
    }
}

impl DefaultForeground for Colour {
    fn default_foreground() -> Self {
        Colour(ansi_colour::Colour::from_code(DEFAULT_FG_ANSI_CODE))
    }
}

impl DefaultBackground for Colour {
    fn default_background() -> Self {
        Colour(ansi_colour::Colour::from_code(DEFAULT_BG_ANSI_CODE))
    }
}

pub type Cell = CommonCell<Colour, Colour>;
