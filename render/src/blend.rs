use rgb24::Rgb24;

pub trait Blend {
    fn blend(current: Rgb24, new: Rgb24) -> Rgb24;
}
