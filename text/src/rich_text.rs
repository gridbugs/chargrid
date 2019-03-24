use crate::default::*;
use crate::wrap::{self, Wrap};
use prototty_render::*;

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct RichTextPartOwned {
    pub text: String,
    pub style: Style,
}

impl From<(String, Style)> for RichTextPartOwned {
    fn from((text, style): (String, Style)) -> Self {
        Self::new(text, style)
    }
}

impl<'a> From<&'a RichTextPartOwned> for RichTextPart<'a> {
    fn from(owned: &'a RichTextPartOwned) -> Self {
        Self {
            text: owned.text.as_str(),
            style: owned.style,
        }
    }
}

impl RichTextPartOwned {
    pub fn new(text: String, style: Style) -> Self {
        Self { text, style }
    }
}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct RichTextPart<'a> {
    pub text: &'a str,
    pub style: Style,
}

impl<'a> RichTextPart<'a> {
    pub fn new(text: &'a str, style: Style) -> Self {
        Self { text, style }
    }
}

impl<'a, S: AsRef<str>> From<&'a (S, Style)> for RichTextPart<'a> {
    fn from(&(ref text, style): &'a (S, Style)) -> Self {
        let text = text.as_ref();
        Self { text, style }
    }
}

impl<'a> From<(&'a str, Style)> for RichTextPart<'a> {
    fn from((text, style): (&'a str, Style)) -> Self {
        Self { text, style }
    }
}

impl<'a> From<&'a str> for RichTextPart<'a> {
    fn from(text: &'a str) -> Self {
        Self {
            text,
            style: DEFAULT_STYLE,
        }
    }
}

impl<'a> From<&'a String> for RichTextPart<'a> {
    fn from(text: &'a String) -> Self {
        let text = text.as_str();
        Self {
            text,
            style: DEFAULT_STYLE,
        }
    }
}

pub struct RichTextView<W: Wrap> {
    wrap: W,
}

impl Default for RichTextView<wrap::Word> {
    fn default() -> Self {
        Self::new(wrap::Word::new())
    }
}

impl<W: Wrap> RichTextView<W> {
    pub fn new(wrap: W) -> Self {
        Self { wrap }
    }
}

impl<'a, T, I, W> View<I> for RichTextView<W>
where
    T: 'a + Into<RichTextPart<'a>> + Copy,
    I: IntoIterator<Item = &'a T>,
    W: Wrap,
{
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        parts: I,
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        self.wrap.clear();
        for part in parts {
            let part: RichTextPart = (*part).into();
            for character in part.text.chars() {
                self.wrap
                    .process_character(character, part.style, context, grid);
            }
        }
        self.wrap.flush(context, grid);
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct RichTextViewSingleLine;

impl RichTextViewSingleLine {
    pub const fn new() -> Self {
        Self
    }
}

impl<'a, T, I> View<I> for RichTextViewSingleLine
where
    T: 'a + Into<RichTextPart<'a>> + Copy,
    I: IntoIterator<Item = &'a T>,
{
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        parts: I,
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        RichTextView::new(wrap::None::new()).view(parts, context, grid)
    }

    fn visible_bounds<R: ViewTransformRgb24>(
        &mut self,
        parts: I,
        _context: ViewContext<R>,
    ) -> Size {
        let width: u32 = parts
            .into_iter()
            .map(|part| {
                let part: RichTextPart = (*part).into();
                part.text.len() as u32
            })
            .sum();
        Size::new(width, 1)
    }
}

pub struct RichStringView<W: Wrap> {
    wrap: W,
}

impl<W: Wrap> RichStringView<W> {
    pub fn new(wrap: W) -> Self {
        Self { wrap }
    }
}

impl<'a, T, W> View<T> for RichStringView<W>
where
    T: 'a + Into<RichTextPart<'a>> + Copy,
    W: Wrap,
{
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        part: T,
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        self.wrap.clear();
        let part: RichTextPart = part.into();
        for character in part.text.chars() {
            self.wrap
                .process_character(character, part.style, context, grid);
        }
        self.wrap.flush(context, grid);
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct RichStringViewSingleLine;

impl RichStringViewSingleLine {
    pub fn new() -> Self {
        Self
    }
}

impl<'a, T> View<T> for RichStringViewSingleLine
where
    T: 'a + Into<RichTextPart<'a>> + Copy,
{
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        part: T,
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        RichStringView::new(wrap::None::new()).view(part, context, grid);
    }

    fn visible_bounds<R: ViewTransformRgb24>(
        &mut self,
        part: T,
        _context: ViewContext<R>,
    ) -> Size {
        let part: RichTextPart = part.into();
        let width = part.text.len() as u32;
        Size::new(width, 1)
    }
}
