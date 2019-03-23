use super::{Coord, Size};
use crate::context::*;
use rgb24::Rgb24;

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

fn set_cell_relative_default<G: ?Sized + ViewGrid, R: ViewTransformRgb24>(
    grid: &mut G,
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
        grid.set_cell_absolute(absolute_coord, absolute_depth, absolute_cell);
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
        set_cell_relative_default(
            self,
            relative_coord,
            relative_depth,
            relative_cell,
            context,
        );
    }
    fn set_cell_absolute(
        &mut self,
        absolute_coord: Coord,
        absolute_depth: i32,
        absolute_cell: ViewCell,
    );

    fn size(&self) -> Size;
}

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

    /// Return the size of the visible component of the element without
    /// rendering it.
    /// By default this is the current context size.
    fn visible_bounds<R: ViewTransformRgb24>(
        &mut self,
        data: T,
        context: ViewContext<R>,
    ) -> Size {
        let _ = data;
        context.size
    }

    /// Render an element and return the size that the element, regardless of the
    /// size of the visible component of the element. This allows decorators to know
    /// the size of the output of a view they are decorating.
    /// By default this calls `view` keeping track of the maximum x and y
    /// components of the relative coords of cells which are set in `grid`.
    fn view_reporting_intended_size<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        data: T,
        context: ViewContext<R>,
        grid: &mut G,
    ) -> Size {
        struct Measure<'a, H> {
            grid: &'a mut H,
            max: Coord,
        }
        impl<'a, H: ViewGrid> ViewGrid for Measure<'a, H> {
            fn set_cell_relative<R: ViewTransformRgb24>(
                &mut self,
                relative_coord: Coord,
                relative_depth: i32,
                relative_cell: ViewCell,
                context: ViewContext<R>,
            ) {
                set_cell_relative_default(
                    self,
                    relative_coord,
                    relative_depth,
                    relative_cell,
                    context,
                );
                self.max.x = self.max.x.max(relative_coord.x);
                self.max.y = self.max.y.max(relative_coord.y);
            }
            fn set_cell_absolute(
                &mut self,
                absolute_coord: Coord,
                absolute_depth: i32,
                absolute_cell: ViewCell,
            ) {
                self.grid.set_cell_absolute(
                    absolute_coord,
                    absolute_depth,
                    absolute_cell,
                );
            }
            fn size(&self) -> Size {
                self.grid.size()
            }
        }
        let mut measure = Measure {
            grid,
            max: Coord::new(0, 0),
        };
        self.view(data, context, &mut measure);
        measure.max.to_size().unwrap() + Size::new_u16(1, 1)
    }
}
