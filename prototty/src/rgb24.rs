#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rgb24 {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Rgb24 {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    pub fn into_u32(self) -> u32 {
        self.red as u32 + ((self.green as u32) << 8) + ((self.blue as u32) << 16)
    }

    pub fn from_u32(u: u32) -> Self {
        Rgb24::new(
            (u & 0xff) as u8,
            ((u >> 8) & 0xff) as u8,
            ((u >> 16) & 16) as u8,
        )
    }
}

pub mod colours {
    use Rgb24;

    pub const BLACK: Rgb24 = Rgb24 {
        red: 0,
        green: 0,
        blue: 0,
    };
    pub const RED: Rgb24 = Rgb24 {
        red: 187,
        green: 0,
        blue: 0,
    };
    pub const GREEN: Rgb24 = Rgb24 {
        red: 0,
        green: 187,
        blue: 0,
    };
    pub const YELLOW: Rgb24 = Rgb24 {
        red: 187,
        green: 187,
        blue: 0,
    };
    pub const BLUE: Rgb24 = Rgb24 {
        red: 0,
        green: 0,
        blue: 187,
    };
    pub const MAGENTA: Rgb24 = Rgb24 {
        red: 187,
        green: 0,
        blue: 187,
    };
    pub const CYAN: Rgb24 = Rgb24 {
        red: 0,
        green: 187,
        blue: 187,
    };
    pub const GREY: Rgb24 = Rgb24 {
        red: 187,
        green: 187,
        blue: 187,
    };

    pub const DARK_GREY: Rgb24 = Rgb24 {
        red: 85,
        green: 85,
        blue: 85,
    };
    pub const BRIGHT_RED: Rgb24 = Rgb24 {
        red: 255,
        green: 85,
        blue: 85,
    };
    pub const BRIGHT_GREEN: Rgb24 = Rgb24 {
        red: 0,
        green: 255,
        blue: 0,
    };
    pub const BRIGHT_YELLOW: Rgb24 = Rgb24 {
        red: 255,
        green: 255,
        blue: 85,
    };
    pub const BRIGHT_BLUE: Rgb24 = Rgb24 {
        red: 85,
        green: 85,
        blue: 255,
    };
    pub const BRIGHT_MAGENTA: Rgb24 = Rgb24 {
        red: 255,
        green: 85,
        blue: 255,
    };
    pub const BRIGHT_CYAN: Rgb24 = Rgb24 {
        red: 85,
        green: 255,
        blue: 255,
    };
    pub const WHITE: Rgb24 = Rgb24 {
        red: 255,
        green: 255,
        blue: 255,
    };
}
