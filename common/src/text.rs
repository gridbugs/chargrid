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

impl<'a> From<(&'a str, Style)> for RichTextPart<'a> {
    fn from((text, style): (&'a str, Style)) -> Self {
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

pub trait Wrap: private_wrap::Sealed {
    #[doc(hidden)]
    fn clear(&mut self);
    #[doc(hidden)]
    fn process_character<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        character: char,
        style: Style,
        context: ViewContext<R>,
        grid: &mut G,
    );
    #[doc(hidden)]
    fn flush<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        let _ = context;
        let _ = grid;
    }
    #[doc(hidden)]
    fn num_lines(&self) -> usize;
}

pub mod wrap {
    use super::Wrap;
    use crate::style::Style;
    use prototty_render::*;

    #[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
    #[derive(Debug, Clone)]
    pub struct None {
        cursor: Coord,
    }

    #[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
    #[derive(Debug, Clone)]
    pub struct Word {
        cursor: Coord,
        current_word_buffer: Vec<ViewCell>,
    }

    #[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
    #[derive(Debug, Clone)]
    pub struct Char {
        cursor: Coord,
    }

    impl None {
        pub fn new() -> Self {
            Self {
                cursor: Coord::new(0, 0),
            }
        }
    }

    impl Word {
        pub fn new() -> Self {
            Self {
                cursor: Coord::new(0, 0),
                current_word_buffer: Vec::new(),
            }
        }
    }

    impl Char {
        pub fn new() -> Self {
            Self {
                cursor: Coord::new(0, 0),
            }
        }
    }

    impl Wrap for None {
        fn clear(&mut self) {
            self.cursor = Coord::new(0, 0);
        }
        fn process_character<G: ViewGrid, R: ViewTransformRgb24>(
            &mut self,
            character: char,
            style: Style,
            context: ViewContext<R>,
            grid: &mut G,
        ) {
            match character {
                '\n' => {
                    self.cursor.x = 0;
                    self.cursor.y += 1;
                }
                '\r' => self.cursor.x = 0,
                other => {
                    let view_cell = style.view_cell(other);
                    grid.set_cell_relative(self.cursor, 0, view_cell, context);
                    self.cursor += Coord::new(1, 0);
                }
            }
        }
        fn num_lines(&self) -> usize {
            self.cursor.y as usize + 1
        }
    }

    impl Wrap for Word {
        fn clear(&mut self) {
            self.cursor = Coord::new(0, 0);
            self.current_word_buffer.clear();
        }

        fn process_character<G: ViewGrid, R: ViewTransformRgb24>(
            &mut self,
            character: char,
            style: Style,
            context: ViewContext<R>,
            grid: &mut G,
        ) {
            match character {
                '\n' => {
                    self.flush(context, grid);
                    self.cursor.x = 0;
                    self.cursor.y += 1;
                }
                '\r' => {
                    self.flush(context, grid);
                    self.cursor.x = 0;
                }
                ' ' => {
                    self.flush(context, grid);
                    if self.cursor.x != 0 {
                        let view_cell = style.view_cell(' ');
                        grid.set_cell_relative(self.cursor, 0, view_cell, context);
                        self.cursor.x += 1;
                        assert!(self.cursor.x <= context.size.width() as i32);
                        if self.cursor.x == context.size.width() as i32 {
                            self.cursor.x = 0;
                            self.cursor.y += 1;
                        }
                    }
                }
                other => {
                    let view_cell = style.view_cell(other);
                    self.current_word_buffer.push(view_cell);
                    assert!(
                        self.cursor.x + self.current_word_buffer.len() as i32
                            <= context.size.width() as i32
                    );
                    if self.cursor.x + self.current_word_buffer.len() as i32
                        == context.size.width() as i32
                    {
                        if self.cursor.x == 0 {
                            self.flush(context, grid);
                        } else {
                            self.cursor.x = 0;
                            self.cursor.y += 1;
                        }
                    }
                }
            }
        }

        fn flush<G: ViewGrid, R: ViewTransformRgb24>(
            &mut self,
            context: ViewContext<R>,
            grid: &mut G,
        ) {
            for view_cell in self.current_word_buffer.drain(..) {
                grid.set_cell_relative(self.cursor, 0, view_cell, context);
                self.cursor.x += 1;
            }
            assert!(self.cursor.x <= context.size.width() as i32);
            if self.cursor.x == context.size.width() as i32 {
                self.cursor.x = 0;
                self.cursor.y += 1;
            }
        }

        fn num_lines(&self) -> usize {
            self.cursor.y as usize + 1
        }
    }

    impl Wrap for Char {
        fn clear(&mut self) {
            self.cursor = Coord::new(0, 0);
        }

        fn process_character<G: ViewGrid, R: ViewTransformRgb24>(
            &mut self,
            character: char,
            style: Style,
            context: ViewContext<R>,
            grid: &mut G,
        ) {
            match character {
                '\n' => {
                    self.cursor.x = 0;
                    self.cursor.y += 1;
                }
                '\r' => self.cursor.x = 0,
                other => {
                    let view_cell = style.view_cell(other);
                    grid.set_cell_relative(self.cursor, 0, view_cell, context);
                    self.cursor += Coord::new(1, 0);
                    if self.cursor.x >= context.size.width() as i32 {
                        self.cursor.x = 0;
                        self.cursor.y += 1;
                    }
                }
            }
        }

        fn num_lines(&self) -> usize {
            self.cursor.y as usize + 1
        }
    }
}

mod private_wrap {
    pub trait Sealed {}
    impl Sealed for super::wrap::None {}
    impl Sealed for super::wrap::Word {}
    impl Sealed for super::wrap::Char {}
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

impl<'a, T, I, W> ViewReportingRenderedSize<I> for RichTextView<W>
where
    T: 'a + Into<RichTextPart<'a>> + Copy,
    I: IntoIterator<Item = &'a T>,
    W: Wrap,
{
    fn view_reporting_render_size<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        parts: I,
        context: ViewContext<R>,
        grid: &mut G,
    ) -> Size {
        grid.set_cell_relative(
            Coord::new(0, 0),
            0,
            ViewCell::new().with_character('@'),
            context,
        );
        self.wrap.clear();
        for part in parts {
            let part: RichTextPart = (*part).into();
            for character in part.text.chars() {
                self.wrap
                    .process_character(character, part.style, context, grid);
            }
        }
        self.wrap.flush(context, grid);
        let num_lines = self.wrap.num_lines();
        Size::new(context.size.width(), num_lines as u32)
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
        self.view_reporting_render_size(parts, context, grid);
    }
}
