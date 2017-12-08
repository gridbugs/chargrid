use prototty::*;
use cgmath::Vector2;
use ansi_colour::Colour;
use defaults::*;

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
pub struct Border<V: View + ViewSize> {
    pub child: V,
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

impl<V: View + ViewSize> Border<V> {
    pub fn new(child: V) -> Self {
        Self {
            child,
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
    pub fn with_title<S: Into<String>>(child: V, title: S) -> Self {
        Self {
            child,
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
    fn child_offset(&self) -> Vector2<i16> {
        Vector2 {
            x: self.padding.left + 1,
            y: self.padding.top + 1,
        }.cast()
    }
    fn span(&self) -> Vector2<i16> {
        (self.child.size() + Vector2 {
            x: self.padding.left + self.padding.right + 1,
            y: self.padding.top + self.padding.bottom + 1,
        }).cast()
    }
}

impl<V: View + ViewSize> View for Border<V> {
    fn view<G: ViewGrid>(&self, offset: Vector2<i16>, depth: i16, grid: &mut G) {
        self.child.view(offset + self.child_offset(), depth, grid);

        let span = self.span();

        if let Some(c) = grid.get_mut(offset) {
            c.update_with_style(self.chars.top_left, depth, self.foreground_colour, self.background_colour,
                                self.bold_border, false);
        }
        if let Some(c) = grid.get_mut(offset + Vector2::new(span.x, 0)) {
            c.update_with_style(self.chars.top_right, depth, self.foreground_colour, self.background_colour,
                                 self.bold_border, false);
        }
        if let Some(c) = grid.get_mut(offset + Vector2::new(0, span.y)) {
            c.update_with_style(self.chars.bottom_left, depth, self.foreground_colour, self.background_colour,
                                 self.bold_border, false);
        }
        if let Some(c) = grid.get_mut(offset + Vector2::new(span.x, span.y)) {
            c.update_with_style(self.chars.bottom_right, depth, self.foreground_colour, self.background_colour,
                                 self.bold_border, false);
        }

        let title_offset = if let Some(title) = self.title.as_ref() {
            let before = offset + Vector2::new(1, 0);
            let after = offset + Vector2::new(title.len() as i16 + 2, 0);

            if let Some(c) = grid.get_mut(before) {
                c.update_with_style(self.chars.before_title, depth, self.foreground_colour, self.background_colour,
                                    self.bold_border, false);
            }
            if let Some(c) = grid.get_mut(after) {
                c.update_with_style(self.chars.after_title, depth, self.foreground_colour, self.background_colour,
                                    self.bold_border, false);
            }

            for (index, ch) in title.chars().enumerate() {
                let coord = offset + Vector2::new(index as i16 + 2, 0);
                if let Some(c) = grid.get_mut(coord) {
                    c.update_with_style(ch, depth, self.title_colour, self.background_colour,
                                        self.bold_title, self.underline_title);
                }
            }

            title.len() as i16 + 2
        } else {
            0
        };

        for i in (1 + title_offset)..span.x {
            if let Some(c) = grid.get_mut(offset + Vector2::new(i, 0)) {
                c.update_with_colour(self.chars.top, depth, self.foreground_colour, self.background_colour);
            }
        }
        for i in 1..span.x {
            if let Some(c) = grid.get_mut(offset + Vector2::new(i, span.y)) {
                c.update_with_colour(self.chars.bottom, depth, self.foreground_colour, self.background_colour);
            }
        }

        for i in 1..span.y {
            if let Some(c) = grid.get_mut(offset + Vector2::new(0, i)) {
                c.update_with_colour(self.chars.left, depth, self.foreground_colour, self.background_colour);
            }
            if let Some(c) = grid.get_mut(offset + Vector2::new(span.x, i)) {
                c.update_with_colour(self.chars.right, depth, self.foreground_colour, self.background_colour);
            }
        }
    }
}

impl<V: View + ViewSize>  ViewSize for Border<V> {
    fn size(&self) -> Vector2<u16> {
        self.child.size() + Vector2 {
            x: self.padding.left + self.padding.right + 2,
            y: self.padding.top + self.padding.bottom + 2,
        }
    }
}
