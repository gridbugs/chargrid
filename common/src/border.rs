use defaults::*;
use prototty_render::*;

/// The characters comprising a border. By default, borders are made of unicode
/// box-drawing characters, but they can be changed to arbitrary characters via
/// this struct.
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct BorderChars {
    pub top: char,
    pub bottom: char,
    pub left: char,
    pub right: char,
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
    pub before_title: char,
    pub after_title: char,
}

impl Default for BorderChars {
    fn default() -> Self {
        Self::single()
    }
}

impl BorderChars {
    pub fn single() -> Self {
        Self {
            top: '─',
            bottom: '─',
            left: '│',
            right: '│',
            top_left: '┌',
            top_right: '┐',
            bottom_left: '└',
            bottom_right: '┘',
            before_title: '┤',
            after_title: '├',
        }
    }
}

/// The space in cells between the edge of the bordered area
/// and the element inside.
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Default, Debug, Clone)]
pub struct BorderPadding {
    pub top: u32,
    pub bottom: u32,
    pub left: u32,
    pub right: u32,
}

/// Decorate another element with a border.
/// It's possible to give the border a title, in which case
/// the text appears in the top-left corner.
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Border {
    pub title: Option<String>,
    pub padding: BorderPadding,
    pub chars: BorderChars,
    pub foreground: Rgb24,
    pub background: Rgb24,
    pub bold: bool,
    pub title_colour: Rgb24,
    pub title_bold: bool,
    pub title_underline: bool,
}

impl Default for Border {
    fn default() -> Self {
        Self {
            title: None,
            padding: Default::default(),
            chars: Default::default(),
            foreground: DEFAULT_FG,
            background: DEFAULT_BG,
            bold: false,
            title_colour: DEFAULT_FG,
            title_bold: false,
            title_underline: false,
        }
    }
}

impl Border {
    pub fn default_with_title<S: Into<String>>(title: S) -> Self {
        Self {
            title: Some(title.into()),
            ..Default::default()
        }
    }
    fn child_offset(&self) -> Coord {
        Coord {
            x: (self.padding.left + 1) as i32,
            y: (self.padding.top + 1) as i32,
        }
    }
    fn span_offset(&self) -> Coord {
        Coord {
            x: (self.padding.left + self.padding.right + 1) as i32,
            y: (self.padding.top + self.padding.bottom + 1) as i32,
        }
    }
    fn view_cell_info(&self, character: char) -> ViewCell {
        ViewCell {
            character: Some(character),
            foreground: Some(self.foreground),
            background: Some(self.background),
            bold: Some(self.bold),
            underline: Some(false),
        }
    }
}

pub struct Bordered<V> {
    pub view: V,
    pub border: Border,
}

impl<T: ?Sized, V: View<T> + ViewSize<T>> View<T> for Bordered<V> {
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        data: &T,
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        self.view
            .view(data, context.add_offset(self.border.child_offset()), grid);
        let span = self.border.span_offset() + self.view.size(data);
        grid.set_cell_relative(
            Coord::new(0, 0),
            0,
            self.border.view_cell_info(self.border.chars.top_left),
            context,
        );
        grid.set_cell_relative(
            Coord::new(span.x, 0),
            0,
            self.border.view_cell_info(self.border.chars.top_right),
            context,
        );
        grid.set_cell_relative(
            Coord::new(0, span.y),
            0,
            self.border.view_cell_info(self.border.chars.bottom_left),
            context,
        );
        grid.set_cell_relative(
            Coord::new(span.x, span.y),
            0,
            self.border.view_cell_info(self.border.chars.bottom_right),
            context,
        );
        let title_offset = if let Some(title) = self.border.title.as_ref() {
            let before = Coord::new(1, 0);
            let after = Coord::new(title.len() as i32 + 2, 0);
            grid.set_cell_relative(
                before,
                0,
                self.border.view_cell_info(self.border.chars.before_title),
                context,
            );
            grid.set_cell_relative(
                after,
                0,
                self.border.view_cell_info(self.border.chars.after_title),
                context,
            );
            for (index, ch) in title.chars().enumerate() {
                let coord = Coord::new(index as i32 + 2, 0);
                grid.set_cell_relative(
                    coord,
                    0,
                    ViewCell {
                        character: Some(ch),
                        bold: Some(self.border.title_bold),
                        underline: Some(self.border.title_underline),
                        foreground: Some(self.border.title_colour),
                        background: Some(self.border.background),
                    },
                    context,
                );
            }
            title.len() as i32 + 2
        } else {
            0
        };
        for i in (1 + title_offset)..span.x {
            grid.set_cell_relative(
                Coord::new(i, 0),
                0,
                self.border.view_cell_info(self.border.chars.top),
                context,
            );
        }
        for i in 1..span.x {
            grid.set_cell_relative(
                Coord::new(i, span.y),
                0,
                self.border.view_cell_info(self.border.chars.bottom),
                context,
            );
        }
        for i in 1..span.y {
            grid.set_cell_relative(
                Coord::new(0, i),
                0,
                self.border.view_cell_info(self.border.chars.left),
                context,
            );
            grid.set_cell_relative(
                Coord::new(span.x, i),
                0,
                self.border.view_cell_info(self.border.chars.right),
                context,
            );
        }
    }
}

impl<T: ?Sized, V: ViewSize<T>> ViewSize<T> for Bordered<V> {
    fn size(&mut self, data: &T) -> Size {
        self.view.size(data) + Size::new(2, 2)
    }
}
