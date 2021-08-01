use chargrid_core::*;

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
    pub foreground: Rgba32,
    pub background: Option<Rgba32>,
    pub bold: bool,
    pub title_style: Style,
}

impl Default for BorderStyle {
    fn default() -> Self {
        Self {
            title: None,
            padding: Default::default(),
            chars: Default::default(),
            foreground: Rgba32::new_grey(255),
            background: None,
            bold: false,
            title_style: Style::default(),
        }
    }
}

impl BorderStyle {
    fn child_offset(&self) -> Coord {
        Coord {
            x: (self.padding.left + 1) as i32,
            y: (self.padding.top + 1) as i32,
        }
    }

    fn child_constrain_size_by(&self) -> Coord {
        Coord::new(
            (self.padding.left + self.padding.right + 2) as i32,
            (self.padding.top + self.padding.bottom + 2) as i32,
        )
    }

    fn span_offset(&self) -> Coord {
        Coord {
            x: (self.padding.left + self.padding.right + 1) as i32,
            y: (self.padding.top + self.padding.bottom + 1) as i32,
        }
    }

    fn render_cell(&self, character: char) -> RenderCell {
        RenderCell {
            character: Some(character),
            style: Style {
                foreground: Some(self.foreground),
                background: self.background,
                bold: Some(self.bold),
                underline: Some(false),
            },
        }
    }

    fn draw_border(&self, size: Size, ctx: Ctx, fb: &mut FrameBuffer) {
        let span = self.span_offset() + size;
        fb.set_cell_relative_to_ctx(
            ctx,
            Coord::new(0, 0),
            0,
            self.render_cell(self.chars.top_left),
        );
        fb.set_cell_relative_to_ctx(
            ctx,
            Coord::new(span.x, 0),
            0,
            self.render_cell(self.chars.top_right),
        );
        fb.set_cell_relative_to_ctx(
            ctx,
            Coord::new(0, span.y),
            0,
            self.render_cell(self.chars.bottom_left),
        );
        fb.set_cell_relative_to_ctx(
            ctx,
            Coord::new(span.x, span.y),
            0,
            self.render_cell(self.chars.bottom_right),
        );
        let title_offset = if let Some(title) = self.title.as_ref() {
            let before = Coord::new(1, 0);
            let after = Coord::new(title.len() as i32 + 2, 0);
            fb.set_cell_relative_to_ctx(ctx, before, 0, self.render_cell(self.chars.before_title));
            fb.set_cell_relative_to_ctx(ctx, after, 0, self.render_cell(self.chars.after_title));
            for (index, ch) in title.chars().enumerate() {
                let coord = Coord::new(index as i32 + 2, 0);
                fb.set_cell_relative_to_ctx(
                    ctx,
                    coord,
                    0,
                    RenderCell {
                        style: self.title_style,
                        character: Some(ch),
                    },
                );
            }
            title.len() as i32 + 2
        } else {
            0
        };
        for i in (1 + title_offset)..span.x {
            fb.set_cell_relative_to_ctx(ctx, Coord::new(i, 0), 0, self.render_cell(self.chars.top));
        }
        for i in 1..span.x {
            fb.set_cell_relative_to_ctx(
                ctx,
                Coord::new(i, span.y),
                0,
                self.render_cell(self.chars.bottom),
            );
        }
        for i in 1..span.y {
            fb.set_cell_relative_to_ctx(
                ctx,
                Coord::new(0, i),
                0,
                self.render_cell(self.chars.left),
            );
            fb.set_cell_relative_to_ctx(
                ctx,
                Coord::new(span.x, i),
                0,
                self.render_cell(self.chars.right),
            );
        }
    }

    fn inner_ctx<'a>(&self, ctx: Ctx<'a>) -> Ctx<'a> {
        ctx.add_offset(self.child_offset())
            .constrain_size_by(self.child_constrain_size_by())
    }
}

pub struct Border<C: Component> {
    pub component: C,
    pub style: BorderStyle,
}

impl<C: Component> Component for Border<C> {
    type Output = C::Output;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        let child_ctx = self.style.inner_ctx(ctx);
        let child_size = self.component.size(state, child_ctx);
        self.component.render(state, child_ctx, fb);
        self.style.draw_border(child_size, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        self.component
            .update(state, self.style.inner_ctx(ctx), event)
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.component.size(state, self.style.inner_ctx(ctx))
            + self.style.child_constrain_size_by().to_size().unwrap()
    }
}
