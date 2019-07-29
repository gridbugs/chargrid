use crate::default::*;
use crate::wrap::{self, Wrap};
use prototty_render::*;

pub struct TextView<W: Wrap> {
    pub style: Style,
    wrap: W,
}

impl<W: Wrap> TextView<W> {
    pub fn new(style: Style, wrap: W) -> Self {
        Self { style, wrap }
    }
    pub fn new_default_style(wrap: W) -> Self {
        Self::new(DEFAULT_STYLE, wrap)
    }
}

impl<S, I, W> View<I> for TextView<W>
where
    S: AsRef<str>,
    I: IntoIterator<Item = S>,
    W: Wrap,
{
    fn view<F: Frame, C: ColModify>(&mut self, parts: I, context: ViewContext<C>, frame: &mut F) {
        self.wrap.clear();
        for part in parts {
            let part = part.as_ref();
            for character in part.chars() {
                self.wrap.process_character(character, self.style, context, frame);
            }
        }
        self.wrap.flush(context, frame);
    }
}

pub struct StringView<W: Wrap> {
    pub style: Style,
    wrap: W,
}

impl<W: Wrap> StringView<W> {
    pub fn new(style: Style, wrap: W) -> Self {
        Self { style, wrap }
    }
    pub fn new_default_style(wrap: W) -> Self {
        Self::new(DEFAULT_STYLE, wrap)
    }
}

impl<'a, S, W> View<S> for StringView<W>
where
    S: AsRef<str>,
    W: Wrap,
{
    fn view<F: Frame, C: ColModify>(&mut self, part: S, context: ViewContext<C>, frame: &mut F) {
        self.wrap.clear();
        let part = part.as_ref();
        for character in part.chars() {
            self.wrap.process_character(character, self.style, context, frame);
        }
        self.wrap.flush(context, frame);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StringViewSingleLine {
    pub style: Style,
}

impl Default for StringViewSingleLine {
    fn default() -> Self {
        Self { style: DEFAULT_STYLE }
    }
}

impl StringViewSingleLine {
    pub const fn new(style: Style) -> Self {
        Self { style }
    }
}

impl<'a, S> View<S> for StringViewSingleLine
where
    S: AsRef<str>,
{
    fn view<F: Frame, C: ColModify>(&mut self, part: S, context: ViewContext<C>, frame: &mut F) {
        StringView::new(self.style, wrap::None::new()).view(part, context, frame);
    }

    fn visible_bounds<C: ColModify>(&mut self, part: S, _context: ViewContext<C>) -> Size {
        let part = part.as_ref();
        let width = part.len() as u32;
        Size::new(width, 1)
    }
}
