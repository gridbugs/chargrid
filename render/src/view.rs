use super::{Coord, Size};
use crate::context::*;
use crate::view_cell::*;

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
            style: Style {
                foreground: relative_cell
                    .style
                    .foreground
                    .map(|rgb24| context.transform_rgb24.transform(rgb24)),
                background: relative_cell
                    .style
                    .background
                    .map(|rgb24| context.transform_rgb24.transform(rgb24)),
                ..relative_cell.style
            },
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
