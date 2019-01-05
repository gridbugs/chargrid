use ansi_colour;
use defaults::*;
use prototty_grid::*;
use prototty_render::Rgb24;

#[derive(Debug, Clone)]
pub struct Colour(pub ansi_colour::Colour);

impl From<Rgb24> for Colour {
    fn from(rgb24: Rgb24) -> Self {
        if rgb24.red == rgb24.green && rgb24.green == rgb24.blue {
            let level = (rgb24.red as u32 * ansi_colour::MAX_GREY_SCALE as u32) / 255;
            return Colour(ansi_colour::Colour::grey_scale(level as u8).unwrap());
        }

        let red = (rgb24.red as u32 * ansi_colour::MAX_RGB_CHANNEL as u32) / 255;
        let green = (rgb24.green as u32 * ansi_colour::MAX_RGB_CHANNEL as u32) / 255;
        let blue = (rgb24.blue as u32 * ansi_colour::MAX_RGB_CHANNEL as u32) / 255;

        Colour(ansi_colour::Colour::rgb(red as u8, green as u8, blue as u8).unwrap())
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
