use super::{Coord, Size};
use crate::col_modify::ColModify;
use crate::context::*;
use crate::view_cell::*;

fn set_cell_relative_to_draw<F: ?Sized + Frame, C: ColModify>(
    frame: &mut F,
    relative_coord: Coord,
    relative_depth: i8,
    relative_cell: ViewCell,
    context: ViewContext<C>,
) {
    if relative_coord.is_valid(context.size) {
        let absolute_coord = relative_coord + context.offset;
        let absolute_depth = relative_depth + context.depth;
        let absolute_cell = ViewCell {
            style: Style {
                foreground: context.col_modify.foreground(relative_cell.style.foreground),
                background: context.col_modify.background(relative_cell.style.background),
                ..relative_cell.style
            },
            ..relative_cell
        };
        frame.set_cell_absolute(absolute_coord, absolute_depth, absolute_cell);
    }
}

fn set_cell_relative_to_measure_size<F: ?Sized + Frame, C: ColModify>(
    frame: &mut F,
    relative_coord: Coord,
    context: ViewContext<C>,
) {
    if relative_coord.is_valid(context.size) {
        let absolute_coord = relative_coord + context.offset;
        const DEFAULT_CELL: ViewCell = ViewCell::new();
        frame.set_cell_absolute(absolute_coord, 0, DEFAULT_CELL);
    }
}

pub trait Frame {
    fn set_cell_relative<C: ColModify>(
        &mut self,
        relative_coord: Coord,
        relative_depth: i8,
        relative_cell: ViewCell,
        context: ViewContext<C>,
    ) {
        set_cell_relative_to_draw(self, relative_coord, relative_depth, relative_cell, context);
    }
    fn set_cell_absolute(&mut self, absolute_coord: Coord, absolute_depth: i8, absolute_cell: ViewCell);
}

struct MeasureBounds {
    max_absolute_coord: Coord,
}

impl MeasureBounds {
    fn new() -> Self {
        Self {
            max_absolute_coord: Coord::new(0, 0),
        }
    }
    fn size(&self, offset: Coord) -> Size {
        (self.max_absolute_coord - offset).to_size().unwrap_or(Size::new(0, 0)) + Size::new(1, 1)
    }
}

impl Frame for MeasureBounds {
    fn set_cell_relative<C: ColModify>(
        &mut self,
        relative_coord: Coord,
        _relative_depth: i8,
        _relative_cell: ViewCell,
        context: ViewContext<C>,
    ) {
        set_cell_relative_to_measure_size(self, relative_coord, context);
    }
    fn set_cell_absolute(&mut self, absolute_coord: Coord, _absolute_depth: i8, _absolute_cell: ViewCell) {
        self.max_absolute_coord.x = self.max_absolute_coord.x.max(absolute_coord.x);
        self.max_absolute_coord.y = self.max_absolute_coord.y.max(absolute_coord.y);
    }
}

pub struct MeasureBoundsAndDraw<'a, D> {
    draw: &'a mut D,
    measure_bounds: MeasureBounds,
}

impl<'a, D> MeasureBoundsAndDraw<'a, D>
where
    D: Frame,
{
    pub fn new(draw: &'a mut D) -> Self {
        Self {
            draw,
            measure_bounds: MeasureBounds::new(),
        }
    }
    fn size(&self, offset: Coord) -> Size {
        self.measure_bounds.size(offset)
    }
}

pub fn measure_size<V, T, C>(view: &mut V, data: T, context: ViewContext<C>) -> Size
where
    V: View<T> + ?Sized,
    C: ColModify,
{
    let mut measure_bounds = MeasureBounds::new();
    view.view(data, context, &mut measure_bounds);
    measure_bounds.size(context.offset)
}

pub fn measure_size_and_draw<V, T, C, F>(view: &mut V, data: T, context: ViewContext<C>, frame: &mut F) -> Size
where
    V: View<T> + ?Sized,
    C: ColModify,
    F: Frame,
{
    let mut measure_bounds_and_draw = MeasureBoundsAndDraw::new(frame);
    view.view(data, context, &mut measure_bounds_and_draw);
    measure_bounds_and_draw.size(context.offset)
}

impl<'a, D> Frame for MeasureBoundsAndDraw<'a, D>
where
    D: Frame,
{
    fn set_cell_relative<C: ColModify>(
        &mut self,
        relative_coord: Coord,
        relative_depth: i8,
        relative_cell: ViewCell,
        context: ViewContext<C>,
    ) {
        self.draw
            .set_cell_relative(relative_coord, relative_depth, relative_cell, context);
        self.measure_bounds
            .set_cell_relative(relative_coord, relative_depth, relative_cell, context);
    }
    fn set_cell_absolute(&mut self, absolute_coord: Coord, absolute_depth: i8, absolute_cell: ViewCell) {
        self.draw
            .set_cell_absolute(absolute_coord, absolute_depth, absolute_cell);
        self.measure_bounds
            .set_cell_absolute(absolute_coord, absolute_depth, absolute_cell);
    }
}

pub trait View<T> {
    fn view<F: Frame, C: ColModify>(&mut self, data: T, context: ViewContext<C>, frame: &mut F);

    fn size<C: ColModify>(&mut self, data: T, context: ViewContext<C>) -> Size {
        measure_size(self, data, context)
    }

    fn view_size<F: Frame, C: ColModify>(&mut self, data: T, context: ViewContext<C>, frame: &mut F) -> Size {
        measure_size_and_draw(self, data, context, frame)
    }
}

impl<'a, T, V: View<T>> View<T> for &'a mut V {
    fn view<F: Frame, C: ColModify>(&mut self, data: T, context: ViewContext<C>, frame: &mut F) {
        (*self).view(data, context, frame)
    }
    fn size<C: ColModify>(&mut self, data: T, context: ViewContext<C>) -> Size {
        (*self).size(data, context)
    }
    fn view_size<F: Frame, C: ColModify>(&mut self, data: T, context: ViewContext<C>, frame: &mut F) -> Size {
        (*self).view_size(data, context, frame)
    }
}
