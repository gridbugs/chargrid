use super::{Coord, Size};
use crate::context::*;
use crate::view_cell::*;

fn set_cell_relative_default<F: ?Sized + Frame, R: ViewTransformRgb24>(
    frame: &mut F,
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
        frame.set_cell_absolute(absolute_coord, absolute_depth, absolute_cell);
    }
}

/// A frame of animation
pub trait Frame {
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
    /// Update the cells in `frame` to describe how a type should be rendered.
    /// This mutably borrows `self` to allow the view to contain buffers/caches which
    /// are updated during rendering.
    fn view<F: Frame, R: ViewTransformRgb24>(
        &mut self,
        data: T,
        context: ViewContext<R>,
        frame: &mut F,
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
    /// components of the relative coords of cells which are set in `frame`.
    fn view_reporting_intended_size<F: Frame, R: ViewTransformRgb24>(
        &mut self,
        data: T,
        context: ViewContext<R>,
        frame: &mut F,
    ) -> Size {
        struct Measure<'a, H> {
            frame: &'a mut H,
            max: Coord,
        }
        impl<'a, H: Frame> Frame for Measure<'a, H> {
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
                self.frame.set_cell_absolute(
                    absolute_coord,
                    absolute_depth,
                    absolute_cell,
                );
            }
            fn size(&self) -> Size {
                self.frame.size()
            }
        }
        let mut measure = Measure {
            frame,
            max: Coord::new(0, 0),
        };
        self.view(data, context, &mut measure);
        measure.max.to_size().unwrap() + Size::new_u16(1, 1)
    }
}

impl<'a, T, V: View<T>> View<T> for &'a mut V {
    fn view<F: Frame, R: ViewTransformRgb24>(
        &mut self,
        data: T,
        context: ViewContext<R>,
        frame: &mut F,
    ) {
        (*self).view(data, context, frame)
    }
}
