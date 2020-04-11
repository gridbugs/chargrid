use crate::default::*;
use crate::wrap::{self, Wrap};
use chargrid_render::*;

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
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn word_wrap_very_wide_context() {
        let mut test_grid = chargrid_test_grid::TestGrid::new(Size::new(6, 1));
        let context = ViewContext::default_with_size(Size::new(Size::max_field(), 10));
        let text = &["hello"];
        let mut text_view = TextView::new_default_style(wrap::Word::new());
        text_view.view(text, context, &mut test_grid);
        assert_eq!(test_grid.string_rows(), &["hello ".to_string()]);
    }

    #[test]
    fn word_wrap_typical_string() {
        let mut test_grid = chargrid_test_grid::TestGrid::new(Size::new(80, 10));
        let context = ViewContext::default_with_size(Size::new(80, 40));
        let text = &[include_str!("sample.txt")];
        let mut text_view = TextView::new_default_style(wrap::Word::new());
        text_view.view(text, context, &mut test_grid);
        assert_eq!(
            test_grid.string_rows(),
            &[
                "Far far away, behind the word mountains, far from the countries Vokalia and     ",
                "Consonantia, there live the blind texts. Separated they live in Bookmarksgrove  ",
                "right at the coast of the Semantics, a large language ocean. A small river      ",
                "named Duden flows by their place and supplies it with the necessary regelialia. ",
                "It is a paradisematic country, in which roasted parts of sentences fly into     ",
                "your mouth.                                                                     ",
                "                                                                                ",
                "Even the all-powerful Pointing has no control about the blind texts it is an    ",
                "almost unorthographic life One day however a small line of blind text by the    ",
                "name of Lorem Ipsum decided to leave for the far World of Grammar. The Big      ",
            ]
        );
    }
}
