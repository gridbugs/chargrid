use crate::style::*;
use prototty_render::*;

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct RichTextPart<'a> {
    pub text: &'a str,
    pub style: Style,
}

impl<'a, S: AsRef<str>> From<&'a (S, Style)> for RichTextPart<'a> {
    fn from(&(ref text, style): &'a (S, Style)) -> Self {
        let text = text.as_ref();
        Self { text, style }
    }
}

impl<'a> From<&'a str> for RichTextPart<'a> {
    fn from(text: &'a str) -> Self {
        Self {
            text,
            style: Default::default(),
        }
    }
}

impl<'a> From<&'a String> for RichTextPart<'a> {
    fn from(text: &'a String) -> Self {
        let text = text.as_str();
        Self {
            text,
            style: Default::default(),
        }
    }
}

pub trait Newline: Copy + private_newline::Sealed {}

pub mod newline {
    #[derive(Clone, Copy)]
    pub struct Ignore;
    #[derive(Clone, Copy)]
    pub struct Crlf;
}

mod private_newline {
    pub enum Newline {
        Ignore,
        Crlf,
    }
    pub trait Sealed {
        fn newline() -> Newline;
    }
    impl Sealed for super::newline::Ignore {
        fn newline() -> Newline {
            Newline::Ignore
        }
    }
    impl Sealed for super::newline::Crlf {
        fn newline() -> Newline {
            Newline::Crlf
        }
    }
}

impl Newline for newline::Ignore {}
impl Newline for newline::Crlf {}

pub trait Wrap: Copy + private_wrap::Sealed {}

pub mod wrap {
    #[derive(Clone, Copy)]
    pub struct None;
    #[derive(Clone, Copy)]
    pub struct Word;
    #[derive(Clone, Copy)]
    pub struct Char;
}

mod private_wrap {
    pub enum Wrap {
        None,
        Word,
        Char,
    }
    pub trait Sealed {
        fn wrap() -> Wrap;
    }
    impl Sealed for super::wrap::None {
        fn wrap() -> Wrap {
            Wrap::None
        }
    }
    impl Sealed for super::wrap::Word {
        fn wrap() -> Wrap {
            Wrap::Word
        }
    }
    impl Sealed for super::wrap::Char {
        fn wrap() -> Wrap {
            Wrap::Char
        }
    }
}

impl Wrap for wrap::None {}
impl Wrap for wrap::Word {}
impl Wrap for wrap::Char {}

pub fn rich_text_part_view<'a, T, N, W, G, R>(
    data: T,
    newline: N,
    wrap: W,
    context: ViewContext<R>,
    grid: &mut G,
) where
    T: Into<RichTextPart<'a>>,
    N: Newline,
    W: Wrap,
    G: ViewGrid,
    R: ViewTransformRgb24,
{
    let part = data.into();
    rich_text_view(&[part], newline, wrap, context, grid)
}

pub fn rich_text_view<'a, T, I, N, W, G, R>(
    parts: I,
    newline: N,
    wrap: W,
    context: ViewContext<R>,
    grid: &mut G,
) where
    T: 'a + Into<RichTextPart<'a>> + Copy,
    I: IntoIterator<Item = &'a T>,
    N: Newline,
    W: Wrap,
    G: ViewGrid,
    R: ViewTransformRgb24,
{
    grid.set_cell_relative(
        Coord::new(0, 0),
        0,
        ViewCell::new().with_character('@'),
        context,
    );
    for part in parts {
        let part: RichTextPart = (*part).into();
    }
}

pub struct RichTextView<N: Newline, W: Wrap> {
    pub newline: N,
    pub wrap: W,
}

impl Default for RichTextView<newline::Crlf, wrap::Word> {
    fn default() -> Self {
        Self::new(newline::Crlf, wrap::Word)
    }
}

impl<N: Newline, W: Wrap> RichTextView<N, W> {
    pub fn new(newline: N, wrap: W) -> Self {
        Self { newline, wrap }
    }
}

impl<'a, T, I, N, W> View<I> for RichTextView<N, W>
where
    T: 'a + Into<RichTextPart<'a>> + Copy,
    I: IntoIterator<Item = &'a T>,
    N: Newline,
    W: Wrap,
{
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        data: I,
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        rich_text_view(data, self.newline, self.wrap, context, grid)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct DummyGrid;

    impl ViewGrid for DummyGrid {
        fn set_cell_absolute(
            &mut self,
            _coord: Coord,
            _depth: i32,
            _view_cell: ViewCell,
        ) {
        }
        fn size(&self) -> Size {
            Size::new(10, 10)
        }
    }

    #[test]
    fn rich_text() {
        let mut grid = DummyGrid;
        single_rich_text_view(
            "foo",
            newline::Crlf,
            wrap::Char,
            ViewContext::default_with_size(Size::new(10, 10)),
            &mut grid,
        );
        single_rich_text_view(
            &"foo".to_string(),
            newline::Crlf,
            wrap::Char,
            ViewContext::default_with_size(Size::new(10, 10)),
            &mut grid,
        );
        rich_text_view(
            &[&"hello".to_string(), &"world".to_string()],
            newline::Crlf,
            wrap::Char,
            ViewContext::default_with_size(Size::new(10, 10)),
            &mut grid,
        );
        rich_text_view(
            &["hello", "world"],
            newline::Crlf,
            wrap::Char,
            ViewContext::default_with_size(Size::new(10, 10)),
            &mut grid,
        );
    }
}
