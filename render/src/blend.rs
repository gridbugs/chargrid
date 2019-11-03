use rgb24::Rgb24;

pub trait Blend: Copy {
    fn blend(self, current: Rgb24, new: Rgb24, alpha: u8) -> Rgb24;
}

pub mod blend_mode {
    use super::*;

    #[derive(Clone, Copy)]
    pub struct Replace;
    impl Blend for Replace {
        fn blend(self, _current: Rgb24, new: Rgb24, _alpha: u8) -> Rgb24 {
            new
        }
    }

    #[derive(Clone, Copy)]
    pub struct LinearInterpolate;
    impl Blend for LinearInterpolate {
        fn blend(self, current: Rgb24, new: Rgb24, alpha: u8) -> Rgb24 {
            current.linear_interpolate(new, alpha)
        }
    }
}
