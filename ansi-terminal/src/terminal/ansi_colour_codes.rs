use prototty_render::Rgb24;

const RGB_START: u8 = 16;
const RGB_MAX_FIELD: u8 = 5;
const RGB_FIELD_RANGE: u8 = RGB_MAX_FIELD + 1;
const RGB_COUNT: u8 = RGB_FIELD_RANGE * RGB_FIELD_RANGE * RGB_FIELD_RANGE;
const RGB_END: u8 = RGB_START + RGB_COUNT - 1;
const GREY_SCALE_START: u8 = RGB_END + 1;
const GREY_SCALE_MAX_LEVEL: u8 = 23;

pub fn nearest_palette_code(Rgb24 { r, g, b }: Rgb24) -> u8 {
    let r = ((r as u16 * RGB_MAX_FIELD as u16) / 255) as u8;
    let g = ((g as u16 * RGB_MAX_FIELD as u16) / 255) as u8;
    let b = ((b as u16 * RGB_MAX_FIELD as u16) / 255) as u8;
    RGB_START + (RGB_FIELD_RANGE * RGB_FIELD_RANGE) * r + RGB_FIELD_RANGE * g + b
}

pub fn nearest_mean_greyscale_code(Rgb24 { r, g, b }: Rgb24) -> u8 {
    let offset = (r as u16 + g as u16 + b as u16) / (GREY_SCALE_MAX_LEVEL as u16 * 3);
    GREY_SCALE_START + offset as u8
}

pub fn nearest_ansi_code(Rgb24 { r, g, b }: Rgb24) -> u8 {
    mod codes {
        pub mod normal {
            pub const BLACK: u8 = 0;
            pub const RED: u8 = 1;
            pub const GREEN: u8 = 2;
            pub const YELLOW: u8 = 3;
            pub const BLUE: u8 = 4;
            pub const MAGENTA: u8 = 5;
            pub const CYAN: u8 = 6;
            pub const GREY: u8 = 7;
        }
        pub mod bright {
            pub const DARK_GREY: u8 = 8;
            pub const RED: u8 = 9;
            pub const GREEN: u8 = 10;
            pub const YELLOW: u8 = 11;
            pub const BLUE: u8 = 12;
            pub const MAGENTA: u8 = 13;
            pub const CYAN: u8 = 14;
            pub const WHITE: u8 = 15;
        }
    }
    const THIRD: u8 = 255 / 3;
    macro_rules! lo {
        () => {
            0..=1
        };
    }
    macro_rules! hi {
        () => {
            2..=3
        };
    }
    match (r / THIRD, g / THIRD, b / THIRD) {
        (0, 0, 0) => codes::normal::BLACK,
        (1, 0, 0) => codes::normal::RED,
        (0, 1, 0) => codes::normal::GREEN,
        (0, 0, 1) => codes::normal::BLUE,
        (1, 1, 0) => codes::normal::YELLOW,
        (0, 1, 1) => codes::normal::CYAN,
        (1, 0, 1) => codes::normal::MAGENTA,
        (1, 1, 1) => codes::bright::DARK_GREY,
        col @ (hi!(), hi!(), hi!()) => {
            if col == (2, 2, 2) {
                // despite not being "bright", this tends to be brighter than dark grey
                codes::normal::GREY
            } else {
                codes::bright::WHITE
            }
        }
        (hi!(), lo!(), lo!()) => codes::bright::RED,
        (lo!(), hi!(), lo!()) => codes::bright::GREEN,
        (lo!(), lo!(), hi!()) => codes::bright::BLUE,
        (hi!(), hi!(), lo!()) => codes::bright::YELLOW,
        (lo!(), hi!(), hi!()) => codes::bright::CYAN,
        (hi!(), lo!(), hi!()) => codes::bright::MAGENTA,
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use prototty_render::Rgb24;

    #[test]
    fn nearest_palette_code_all_cases() {
        for r in 0..=255 {
            for g in 0..=255 {
                for b in 0..=255 {
                    let c = Rgb24::new(r, g, b);
                    let _ = super::nearest_palette_code(c);
                }
            }
        }
    }

    #[test]
    fn nearest_ansi_code_all_cases() {
        for r in 0..=255 {
            for g in 0..=255 {
                for b in 0..=255 {
                    let c = Rgb24::new(r, g, b);
                    let _ = super::nearest_ansi_code(c);
                }
            }
        }
    }

    #[test]
    fn nearest_mean_greyscale_code_all_cases() {
        for r in 0..=255 {
            for g in 0..=255 {
                for b in 0..=255 {
                    let c = Rgb24::new(r, g, b);
                    let _ = super::nearest_mean_greyscale_code(c);
                }
            }
        }
    }
}
