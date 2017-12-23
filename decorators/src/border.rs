use prototty_traits::*;
use prototty_defaults::*;
use super::Decorated;

/// The characters comprising a border. By default, borders are made of unicode
/// box-drawing characters, but they can be changed to arbitrary characters via
/// this struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct BorderPadding {
    pub top: u16,
    pub bottom: u16,
    pub left: u16,
    pub right: u16,
}

/// Decorate another element with a border.
/// The child element must implement `View` and `ViewSize`,
/// and can be accessed via the `child` field.
/// It's possible to give the border a title, in which case
/// the text appears in the top-left corner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Border {
    pub title: Option<String>,
    pub padding: BorderPadding,
    pub chars: BorderChars,
    pub foreground_colour: Colour,
    pub background_colour: Colour,
    pub title_colour: Colour,
    pub bold_title: bool,
    pub underline_title: bool,
    pub bold_border: bool,
}

impl<'a, 'b, T, V: View<T> + ViewSize<T>> View<T> for Decorated<'a, 'b, V, Border> {
    fn view<G: ViewGrid>(&self, value: &T, offset: Coord, depth: i16, grid: &mut G) {

        self.view.view(value, offset + self.decorator.child_offset(), depth, grid);

        let span = self.decorator.span_offset() + self.view.size(value).cast().unwrap();

        if let Some(c) = grid.get_mut(offset) {
            c.update_with_style(self.decorator.chars.top_left, depth, self.decorator.foreground_colour, self.decorator.background_colour,
                                self.decorator.bold_border, false);
        }
        if let Some(c) = grid.get_mut(offset + Coord::new(span.x, 0)) {
            c.update_with_style(self.decorator.chars.top_right, depth, self.decorator.foreground_colour, self.decorator.background_colour,
                                 self.decorator.bold_border, false);
        }
        if let Some(c) = grid.get_mut(offset + Coord::new(0, span.y)) {
            c.update_with_style(self.decorator.chars.bottom_left, depth, self.decorator.foreground_colour, self.decorator.background_colour,
                                 self.decorator.bold_border, false);
        }
        if let Some(c) = grid.get_mut(offset + Coord::new(span.x, span.y)) {
            c.update_with_style(self.decorator.chars.bottom_right, depth, self.decorator.foreground_colour, self.decorator.background_colour,
                                 self.decorator.bold_border, false);
        }

        let title_offset = if let Some(title) = self.decorator.title.as_ref() {
            let before = offset + Coord::new(1, 0);
            let after = offset + Coord::new(title.len() as i16 + 2, 0);

            if let Some(c) = grid.get_mut(before) {
                c.update_with_style(self.decorator.chars.before_title, depth, self.decorator.foreground_colour, self.decorator.background_colour,
                                    self.decorator.bold_border, false);
            }
            if let Some(c) = grid.get_mut(after) {
                c.update_with_style(self.decorator.chars.after_title, depth, self.decorator.foreground_colour, self.decorator.background_colour,
                                    self.decorator.bold_border, false);
            }

            for (index, ch) in title.chars().enumerate() {
                let coord = offset + Coord::new(index as i16 + 2, 0);
                if let Some(c) = grid.get_mut(coord) {
                    c.update_with_style(ch, depth, self.decorator.title_colour, self.decorator.background_colour,
                                        self.decorator.bold_title, self.decorator.underline_title);
                }
            }

            title.len() as i16 + 2
        } else {
            0
        };

        for i in (1 + title_offset)..span.x {
            if let Some(c) = grid.get_mut(offset + Coord::new(i, 0)) {
                c.update_with_style(self.decorator.chars.top, depth, self.decorator.foreground_colour, self.decorator.background_colour,
                                    self.decorator.bold_border, false);
            }
        }
        for i in 1..span.x {
            if let Some(c) = grid.get_mut(offset + Coord::new(i, span.y)) {
                c.update_with_style(self.decorator.chars.bottom, depth, self.decorator.foreground_colour, self.decorator.background_colour,
                                    self.decorator.bold_border, false);
            }
        }

        for i in 1..span.y {
            if let Some(c) = grid.get_mut(offset + Coord::new(0, i)) {
                c.update_with_style(self.decorator.chars.left, depth, self.decorator.foreground_colour, self.decorator.background_colour,
                                    self.decorator.bold_border, false);
            }
            if let Some(c) = grid.get_mut(offset + Coord::new(span.x, i)) {
                c.update_with_style(self.decorator.chars.right, depth, self.decorator.foreground_colour, self.decorator.background_colour,
                                    self.decorator.bold_border, false);
            }
        }
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
            x: (self.padding.left + 1) as i16,
            y: (self.padding.top + 1) as i16,
        }
    }
    fn span_offset(&self) -> Coord {
        Coord {
            x: (self.padding.left + self.padding.right + 1) as i16,
            y: (self.padding.top + self.padding.bottom + 1) as i16,
        }
    }
}

#[test]
fn example() {
    struct DummyCell;
    impl ViewCell for  DummyCell {
        fn update(&mut self, _: char, _: i16) {}
        fn update_with_colour(&mut self, _: char, _: i16, _: Colour, _: Colour) {}
        fn update_with_style(&mut self, _: char, _: i16, _: Colour, _: Colour,
                             _: bool, _: bool) {}
    }
    struct DummyGrid;
    impl ViewGrid for DummyGrid {
        type Cell = DummyCell;
        fn get_mut(&mut self, _: Coord) -> Option<&mut Self::Cell> { None }
    }

    struct A;
    impl View<()> for A {
        fn view<G: ViewGrid>(&self, _: &(), _: Coord, _: i16, _: &mut G) {}
    }
    impl ViewSize<()> for A {
        fn size(&self, _: &()) -> Size { (0, 0).into() }
    }

    A.view(&(), Coord::new(0, 0), 0, &mut DummyGrid);
    let border = Border::new();
    Decorated::new(&A, &border).view(&(), Coord::new(0, 0), 0, &mut DummyGrid);
}
