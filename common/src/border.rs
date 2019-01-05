use decorated::Decorated;
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
    pub foreground_colour: Rgb24,
    pub background_colour: Rgb24,
    pub title_colour: Rgb24,
    pub bold_title: bool,
    pub underline_title: bool,
    pub bold_border: bool,
}

impl<T: ?Sized, V: View<T> + ViewSize<T>> View<T> for Decorated<V, Border> {
    fn view<G: ViewGrid>(&mut self, data: &T, offset: Coord, depth: i32, grid: &mut G) {
        self.view
            .view(data, offset + self.decorator.child_offset(), depth, grid);

        let span = self.decorator.span_offset() + self.view.size(data);

        if let Some(c) = grid.get_mut(offset, depth) {
            c.set_character(self.decorator.chars.top_left);
            self.decorator.write_border_style(c);
        }
        if let Some(c) = grid.get_mut(offset + Coord::new(span.x, 0), depth) {
            c.set_character(self.decorator.chars.top_right);
            self.decorator.write_border_style(c);
        }
        if let Some(c) = grid.get_mut(offset + Coord::new(0, span.y), depth) {
            c.set_character(self.decorator.chars.bottom_left);
            self.decorator.write_border_style(c);
        }
        if let Some(c) = grid.get_mut(offset + Coord::new(span.x, span.y), depth) {
            c.set_character(self.decorator.chars.bottom_right);
            self.decorator.write_border_style(c);
        }

        let title_offset = if let Some(title) = self.decorator.title.as_ref() {
            let before = offset + Coord::new(1, 0);
            let after = offset + Coord::new(title.len() as i32 + 2, 0);

            if let Some(c) = grid.get_mut(before, depth) {
                c.set_character(self.decorator.chars.before_title);
                self.decorator.write_border_style(c);
            }
            if let Some(c) = grid.get_mut(after, depth) {
                c.set_character(self.decorator.chars.after_title);
                self.decorator.write_border_style(c);
            }

            for (index, ch) in title.chars().enumerate() {
                let coord = offset + Coord::new(index as i32 + 2, 0);
                if let Some(c) = grid.get_mut(coord, depth) {
                    c.set_character(ch);
                    c.set_foreground_colour(self.decorator.title_colour);
                    c.set_background_colour(self.decorator.background_colour);
                    c.set_bold(self.decorator.bold_title);
                    c.set_underline(self.decorator.underline_title);
                }
            }

            title.len() as i32 + 2
        } else {
            0
        };

        for i in (1 + title_offset)..span.x {
            if let Some(c) = grid.get_mut(offset + Coord::new(i, 0), depth) {
                c.set_character(self.decorator.chars.top);
                self.decorator.write_border_style(c);
            }
        }
        for i in 1..span.x {
            if let Some(c) = grid.get_mut(offset + Coord::new(i, span.y), depth) {
                c.set_character(self.decorator.chars.bottom);
                self.decorator.write_border_style(c);
            }
        }

        for i in 1..span.y {
            if let Some(c) = grid.get_mut(offset + Coord::new(0, i), depth) {
                c.set_character(self.decorator.chars.left);
                self.decorator.write_border_style(c);
            }
            if let Some(c) = grid.get_mut(offset + Coord::new(span.x, i), depth) {
                c.set_character(self.decorator.chars.right);
                self.decorator.write_border_style(c);
            }
        }
    }
}

impl<T: ?Sized, V: View<T> + ViewSize<T>> ViewSize<T> for Decorated<V, Border> {
    fn size(&mut self, data: &T) -> Size {
        self.view.size(data) + Size::new(2, 2)
    }
}

impl Border {
    pub fn new() -> Self {
        Self {
            title: None,
            padding: Default::default(),
            chars: Default::default(),
            foreground_colour: DEFAULT_FG,
            background_colour: DEFAULT_BG,
            title_colour: DEFAULT_FG,
            bold_title: false,
            underline_title: false,
            bold_border: false,
        }
    }
    pub fn with_title<S: Into<String>>(title: S) -> Self {
        Self {
            title: Some(title.into()),
            padding: Default::default(),
            chars: Default::default(),
            foreground_colour: DEFAULT_FG,
            background_colour: DEFAULT_BG,
            title_colour: DEFAULT_FG,
            bold_title: false,
            underline_title: false,
            bold_border: false,
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
    fn write_border_style<C: ViewCell>(&self, cell: &mut C) {
        cell.set_foreground_colour(self.foreground_colour);
        cell.set_background_colour(self.background_colour);
        cell.set_bold(self.bold_border);
        cell.set_underline(false);
    }
}
