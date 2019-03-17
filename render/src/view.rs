use super::{Coord, Size};
use rgb24::Rgb24;

pub trait ViewTransformRgb24: Copy {
    fn transform(&self, rgb24: Rgb24) -> Rgb24;
}

impl<F: Fn(Rgb24) -> Rgb24 + Copy> ViewTransformRgb24 for F {
    fn transform(&self, rgb24: Rgb24) -> Rgb24 {
        (self)(rgb24)
    }
}

#[derive(Clone, Copy)]
pub struct ViewTransformRgb24Identity;

impl ViewTransformRgb24 for ViewTransformRgb24Identity {
    fn transform(&self, rgb24: Rgb24) -> Rgb24 {
        rgb24
    }
}

#[derive(Clone, Copy)]
pub struct ViewTransformRgb24Compose<Inner: ViewTransformRgb24, Outer: ViewTransformRgb24>
{
    inner: Inner,
    outer: Outer,
}

impl<Inner, Outer> ViewTransformRgb24 for ViewTransformRgb24Compose<Inner, Outer>
where
    Inner: ViewTransformRgb24,
    Outer: ViewTransformRgb24,
{
    fn transform(&self, rgb24: Rgb24) -> Rgb24 {
        self.outer.transform(self.inner.transform(rgb24))
    }
}

#[derive(Clone, Copy)]
pub struct ViewContext<R: ViewTransformRgb24 = ViewTransformRgb24Identity> {
    pub inner_offset: Coord,
    pub outer_offset: Coord,
    pub depth: i32,
    pub transform_rgb24: R,
    pub size: Size,
}

impl ViewContext<ViewTransformRgb24Identity> {
    pub fn default_with_size(size: Size) -> Self {
        Self {
            inner_offset: Coord::new(0, 0),
            outer_offset: Coord::new(0, 0),
            depth: 0,
            transform_rgb24: ViewTransformRgb24Identity,
            size,
        }
    }
}

impl<R: ViewTransformRgb24> ViewContext<R> {
    pub fn new(
        inner_offset: Coord,
        outer_offset: Coord,
        depth: i32,
        transform_rgb24: R,
        size: Size,
    ) -> Self {
        Self {
            inner_offset,
            outer_offset,
            depth,
            transform_rgb24,
            size,
        }
    }

    pub fn add_inner_offset(self, offset_delta: Coord) -> Self {
        Self {
            inner_offset: self.inner_offset + offset_delta,
            ..self
        }
    }

    pub fn add_offset(self, offset_delta: Coord) -> Self {
        Self {
            outer_offset: self.outer_offset + offset_delta,
            size: (self.size.to_coord().unwrap() - offset_delta)
                .to_size()
                .unwrap_or(Size::new_u16(0, 0)),
            ..self
        }
    }

    pub fn add_depth(self, depth_delta: i32) -> Self {
        Self {
            depth: self.depth + depth_delta,
            ..self
        }
    }

    pub fn constrain_size_to(self, size: Size) -> Self {
        Self {
            size: Size::new(self.size.x().min(size.x()), self.size.y().min(size.y())),
            ..self
        }
    }

    pub fn constrain_size_by(self, size: Size) -> Self {
        Self {
            size: self.size.saturating_sub(size),
            ..self
        }
    }

    pub fn compose_transform_rgb24<Inner: ViewTransformRgb24>(
        self,
        inner: Inner,
    ) -> ViewContext<ViewTransformRgb24Compose<Inner, R>> {
        ViewContext {
            transform_rgb24: ViewTransformRgb24Compose {
                inner,
                outer: self.transform_rgb24,
            },
            inner_offset: self.inner_offset,
            outer_offset: self.outer_offset,
            depth: self.depth,
            size: self.size,
        }
    }
}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ViewCell {
    pub character: Option<char>,
    pub bold: Option<bool>,
    pub underline: Option<bool>,
    pub foreground: Option<Rgb24>,
    pub background: Option<Rgb24>,
}

impl ViewCell {
    pub const fn new() -> Self {
        Self {
            character: None,
            bold: None,
            underline: None,
            foreground: None,
            background: None,
        }
    }
    pub const fn with_character(self, character: char) -> Self {
        Self {
            character: Some(character),
            ..self
        }
    }
    pub const fn with_bold(self, bold: bool) -> Self {
        Self {
            bold: Some(bold),
            ..self
        }
    }
    pub const fn with_underline(self, underline: bool) -> Self {
        Self {
            underline: Some(underline),
            ..self
        }
    }
    pub const fn with_foreground(self, foreground: Rgb24) -> Self {
        Self {
            foreground: Some(foreground),
            ..self
        }
    }
    pub const fn with_background(self, background: Rgb24) -> Self {
        Self {
            background: Some(background),
            ..self
        }
    }
    pub fn set_character(&mut self, character: char) {
        self.character = Some(character);
    }
    pub fn clear_character(&mut self) {
        self.character = None;
    }
    pub fn set_bold(&mut self, bold: bool) {
        self.bold = Some(bold);
    }
    pub fn clear_bold(&mut self) {
        self.bold = None;
    }
    pub fn set_underline(&mut self, underline: bool) {
        self.underline = Some(underline);
    }
    pub fn clear_underline(&mut self) {
        self.underline = None;
    }
    pub fn set_foreground(&mut self, foreground: Rgb24) {
        self.foreground = Some(foreground);
    }
    pub fn clear_foreground(&mut self) {
        self.foreground = None;
    }
    pub fn set_background(&mut self, background: Rgb24) {
        self.background = Some(background);
    }
    pub fn clear_background(&mut self) {
        self.background = None;
    }
    pub fn coalesce(self, other: Self) -> Self {
        Self {
            character: (self.character.or(other.character)),
            bold: (self.bold.or(other.bold)),
            underline: (self.underline.or(other.underline)),
            foreground: (self.foreground.or(other.foreground)),
            background: (self.background.or(other.background)),
        }
    }
}

/// A grid of cells
pub trait ViewGrid {
    fn set_cell_relative<R: ViewTransformRgb24>(
        &mut self,
        relative_coord: Coord,
        relative_depth: i32,
        relative_cell: ViewCell,
        context: ViewContext<R>,
    ) {
        let adjusted_relative_coord = relative_coord + context.inner_offset;
        if adjusted_relative_coord.is_valid(context.size) {
            let absolute_coord = adjusted_relative_coord + context.outer_offset;
            let absolute_depth = relative_depth + context.depth;
            let absolute_cell = ViewCell {
                foreground: relative_cell
                    .foreground
                    .map(|rgb24| context.transform_rgb24.transform(rgb24)),
                background: relative_cell
                    .background
                    .map(|rgb24| context.transform_rgb24.transform(rgb24)),
                ..relative_cell
            };
            self.set_cell_absolute(absolute_coord, absolute_depth, absolute_cell);
        }
    }

    fn set_cell_absolute(
        &mut self,
        absolute_coord: Coord,
        absolute_depth: i32,
        absolute_cell: ViewCell,
    );

    fn size(&self) -> Size;
}

/// Defines a method for rendering a `T` to the terminal.
pub trait View<T> {
    /// Update the cells in `grid` to describe how a type should be rendered.
    /// This mutably borrows `self` to allow the view to contain buffers/caches which
    /// are updated during rendering.
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        data: T,
        context: ViewContext<R>,
        grid: &mut G,
    );
}

/// Report the size of a `T` when rendered.
pub trait ViewSize<T> {
    /// Returns the size in cells of the rectangle containing a ui element.
    /// This allows for the implementation of decorator ui components that
    /// render a border around some inner element.
    fn size(&mut self, data: T) -> Size;
}

pub trait ViewReportingRenderedSize<T> {
    fn view_reporting_render_size<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        data: T,
        context: ViewContext<R>,
        grid: &mut G,
    ) -> Size;
}
