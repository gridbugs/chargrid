use defaults::*;
use prototty_render::*;

/// The characters comprising a border. By default, borders are made of unicode
/// box-drawing characters, but they can be changed to arbitrary characters via
/// this struct.
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
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
#[derive(Default, Debug, Clone, Copy)]
pub struct BorderPadding {
    pub top: u32,
    pub bottom: u32,
    pub left: u32,
    pub right: u32,
}

impl BorderPadding {
    pub fn all(padding: u32) -> Self {
        Self {
            top: padding,
            bottom: padding,
            left: padding,
            right: padding,
        }
    }
}

/// Decorate another element with a border.
/// It's possible to give the border a title, in which case
/// the text appears in the top-left corner.
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct BorderStyle {
    pub title: Option<String>,
    pub padding: BorderPadding,
    pub chars: BorderChars,
    pub foreground: Rgb24,
    pub background: Rgb24,
    pub bold: bool,
    pub title_style: Style,
}

impl Default for BorderStyle {
    fn default() -> Self {
        Self {
            title: None,
            padding: Default::default(),
            chars: Default::default(),
            foreground: DEFAULT_FG,
            background: DEFAULT_BG,
            bold: false,
            title_style: Default::default(),
        }
    }
}

impl BorderStyle {
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
    fn child_constrain_size_by(&self) -> Size {
        Size::new(
            self.padding.left + self.padding.right + 2,
            self.padding.top + self.padding.bottom + 2,
        )
    }
    fn span_offset(&self) -> Coord {
        Coord {
            x: (self.padding.left + self.padding.right + 1) as i32,
            y: (self.padding.top + self.padding.bottom + 1) as i32,
        }
    }
    fn view_cell(&self, character: char) -> ViewCell {
        ViewCell {
            character: Some(character),
            style: Style {
                foreground: Some(self.foreground),
                background: Some(self.background),
                bold: Some(self.bold),
                underline: Some(false),
            },
        }
    }
}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct BorderView<V> {
    pub view: V,
}

impl<V> BorderView<V> {
    pub fn new(view: V) -> Self {
        Self { view }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BorderData<'a, T> {
    pub style: &'a BorderStyle,
    pub data: T,
}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct BorderWithOwnedStyleData<T> {
    pub style: BorderStyle,
    pub data: T,
}

fn draw_border<F, C>(style: &BorderStyle, size: Size, context: ViewContext<C>, frame: &mut F)
where
    C: ColModify,
    F: Frame,
{
    let span = style.span_offset() + size;
    frame.set_cell_relative(Coord::new(0, 0), 0, style.view_cell(style.chars.top_left), context);
    frame.set_cell_relative(
        Coord::new(span.x, 0),
        0,
        style.view_cell(style.chars.top_right),
        context,
    );
    frame.set_cell_relative(
        Coord::new(0, span.y),
        0,
        style.view_cell(style.chars.bottom_left),
        context,
    );
    frame.set_cell_relative(
        Coord::new(span.x, span.y),
        0,
        style.view_cell(style.chars.bottom_right),
        context,
    );
    let title_offset = if let Some(title) = style.title.as_ref() {
        let before = Coord::new(1, 0);
        let after = Coord::new(title.len() as i32 + 2, 0);
        frame.set_cell_relative(before, 0, style.view_cell(style.chars.before_title), context);
        frame.set_cell_relative(after, 0, style.view_cell(style.chars.after_title), context);
        for (index, ch) in title.chars().enumerate() {
            let coord = Coord::new(index as i32 + 2, 0);
            frame.set_cell_relative(
                coord,
                0,
                ViewCell {
                    style: style.title_style,
                    character: Some(ch),
                },
                context,
            );
        }
        title.len() as i32 + 2
    } else {
        0
    };
    for i in (1 + title_offset)..span.x {
        frame.set_cell_relative(Coord::new(i, 0), 0, style.view_cell(style.chars.top), context);
    }
    for i in 1..span.x {
        frame.set_cell_relative(Coord::new(i, span.y), 0, style.view_cell(style.chars.bottom), context);
    }
    for i in 1..span.y {
        frame.set_cell_relative(Coord::new(0, i), 0, style.view_cell(style.chars.left), context);
        frame.set_cell_relative(Coord::new(span.x, i), 0, style.view_cell(style.chars.right), context);
    }
}

fn border_view<V, T, F, C>(view: &mut V, data: T, style: &BorderStyle, context: ViewContext<C>, frame: &mut F)
where
    V: View<T>,
    T: Clone,
    C: ColModify,
    F: Frame,
{
    let child_context = context
        .add_offset(style.child_offset())
        .constrain_size_by(style.child_constrain_size_by());
    view.view(data.clone(), child_context, frame);
    let size = view.visible_bounds(data, child_context);
    draw_border(style, size, context, frame);
}

impl<'a, T, V: View<&'a T>> View<&'a BorderWithOwnedStyleData<T>> for BorderView<V> {
    fn view<F: Frame, C: ColModify>(
        &mut self,
        BorderWithOwnedStyleData { style, data }: &'a BorderWithOwnedStyleData<T>,
        context: ViewContext<C>,
        frame: &mut F,
    ) {
        self.view(BorderData { style, data }, context, frame);
    }
    fn visible_bounds<C: ColModify>(
        &mut self,
        BorderWithOwnedStyleData { style, data }: &'a BorderWithOwnedStyleData<T>,
        context: ViewContext<C>,
    ) -> Size {
        self.visible_bounds(BorderData { style, data }, context)
    }
}

impl<'a, T: Clone, V: View<T>> View<BorderData<'a, T>> for BorderView<V> {
    fn view<F: Frame, C: ColModify>(
        &mut self,
        BorderData { style, data }: BorderData<'a, T>,
        context: ViewContext<C>,
        frame: &mut F,
    ) {
        border_view(&mut self.view, data, style, context, frame);
    }

    fn visible_bounds<C: ColModify>(
        &mut self,
        BorderData { style, data }: BorderData<'a, T>,
        context: ViewContext<C>,
    ) -> Size {
        let bounds_of_child_with_border = self.view.visible_bounds(data, context) + style.child_constrain_size_by();
        let x = bounds_of_child_with_border.x().min(context.size.x());
        let y = bounds_of_child_with_border.y().min(context.size.y());
        Size::new(x, y)
    }
}

pub struct BorderView_<'v, 's, V> {
    pub view: &'v mut V,
    pub style: &'s BorderStyle,
}

impl<'v, 's, V, T> View<T> for BorderView_<'v, 's, V>
where
    V: View<T>,
    T: Clone,
{
    fn view<F: Frame, C: ColModify>(&mut self, data: T, context: ViewContext<C>, frame: &mut F) {
        border_view(self.view, data, self.style, context, frame);
    }
    fn visible_bounds<C: ColModify>(&mut self, data: T, context: ViewContext<C>) -> Size {
        let bounds_of_child_with_border =
            self.view.visible_bounds(data, context) + self.style.child_constrain_size_by();
        let x = bounds_of_child_with_border.x().min(context.size.x());
        let y = bounds_of_child_with_border.y().min(context.size.y());
        Size::new(x, y)
    }
}
