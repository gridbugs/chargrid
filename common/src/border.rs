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

        grid.set_cell(
            offset,
            depth,
            self.decorator.view_cell_info(self.decorator.chars.top_left),
        );
        grid.set_cell(
            offset + Coord::new(span.x, 0),
            depth,
            self.decorator
                .view_cell_info(self.decorator.chars.top_right),
        );
        grid.set_cell(
            offset + Coord::new(0, span.y),
            depth,
            self.decorator
                .view_cell_info(self.decorator.chars.bottom_left),
        );
        grid.set_cell(
            offset + Coord::new(span.x, span.y),
            depth,
            self.decorator
                .view_cell_info(self.decorator.chars.bottom_right),
        );
        let title_offset = if let Some(title) = self.decorator.title.as_ref() {
            let before = offset + Coord::new(1, 0);
            let after = offset + Coord::new(title.len() as i32 + 2, 0);
            grid.set_cell(
                before,
                depth,
                self.decorator
                    .view_cell_info(self.decorator.chars.before_title),
            );
            grid.set_cell(
                after,
                depth,
                self.decorator
                    .view_cell_info(self.decorator.chars.after_title),
            );
            for (index, ch) in title.chars().enumerate() {
                let coord = offset + Coord::new(index as i32 + 2, 0);
                grid.set_cell(
                    coord,
                    depth,
                    ViewCellInfo {
                        character: Some(ch),
                        bold: Some(self.decorator.bold_title),
                        underline: Some(self.decorator.underline_title),
                        foreground: Some(self.decorator.title_colour),
                        background: Some(self.decorator.background_colour),
                    },
                );
            }

            title.len() as i32 + 2
        } else {
            0
        };

        for i in (1 + title_offset)..span.x {
            grid.set_cell(
                offset + Coord::new(i, 0),
                depth,
                self.decorator.view_cell_info(self.decorator.chars.top),
            );
        }
        for i in 1..span.x {
            grid.set_cell(
                offset + Coord::new(i, span.y),
                depth,
                self.decorator.view_cell_info(self.decorator.chars.bottom),
            );
        }

        for i in 1..span.y {
            grid.set_cell(
                offset + Coord::new(0, i),
                depth,
                self.decorator.view_cell_info(self.decorator.chars.left),
            );
            grid.set_cell(
                offset + Coord::new(span.x, i),
                depth,
                self.decorator.view_cell_info(self.decorator.chars.right),
            );
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
    fn view_cell_info(&self, character: char) -> ViewCellInfo {
        ViewCellInfo {
            character: Some(character),
            foreground: Some(self.foreground_colour),
            background: Some(self.background_colour),
            bold: Some(self.bold_border),
            underline: Some(false),
        }
    }
}
