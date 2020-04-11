use crate::wrap::{self, Wrap};
use chargrid_render::*;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct RichTextPartOwned {
    pub text: String,
    pub style: Style,
}

impl RichTextPartOwned {
    pub fn new(text: String, style: Style) -> Self {
        Self { text, style }
    }
    pub fn as_rich_text_part(&self) -> RichTextPart {
        RichTextPart {
            text: self.text.as_str(),
            style: self.style,
        }
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

impl<'a, I, W> View<I> for RichTextView<W>
where
    I: IntoIterator<Item = RichTextPart<'a>>,
    W: Wrap,
{
    fn view<F: Frame, C: ColModify>(&mut self, parts: I, context: ViewContext<C>, frame: &mut F) {
        self.wrap.clear();
        for part in parts {
            for character in part.text.chars() {
                self.wrap.process_character(character, part.style, context, frame);
            }
        }
        self.wrap.flush(context, frame);
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct RichTextViewSingleLine;

impl RichTextViewSingleLine {
    pub const fn new() -> Self {
        Self
    }
}

impl<'a, I> View<I> for RichTextViewSingleLine
where
    I: IntoIterator<Item = RichTextPart<'a>>,
{
    fn view<F: Frame, C: ColModify>(&mut self, parts: I, context: ViewContext<C>, frame: &mut F) {
        RichTextView::new(wrap::None::new()).view(parts, context, frame)
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

impl<'a, W> View<RichTextPart<'a>> for RichStringView<W>
where
    W: Wrap,
{
    fn view<F: Frame, C: ColModify>(&mut self, part: RichTextPart<'a>, context: ViewContext<C>, frame: &mut F) {
        self.wrap.clear();
        let part: RichTextPart = part.into();
        for character in part.text.chars() {
            self.wrap.process_character(character, part.style, context, frame);
        }
        self.wrap.flush(context, frame);
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct RichStringViewSingleLine;

impl RichStringViewSingleLine {
    pub fn new() -> Self {
        Self
    }
}

impl<'a> View<RichTextPart<'a>> for RichStringViewSingleLine {
    fn view<F: Frame, C: ColModify>(&mut self, part: RichTextPart<'a>, context: ViewContext<C>, frame: &mut F) {
        RichStringView::new(wrap::None::new()).view(part, context, frame);
    }
}
