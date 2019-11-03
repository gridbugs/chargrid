use prototty_render::*;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct VerticalScrollBarStyle {
    pub style: Style,
    pub character: char,
    pub left_padding: u32,
}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct VerticalScrollLimits {
    last_rendered_inner_height: u32,
    last_rendered_outer_height: u32,
}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct VerticalScrollState {
    scroll_position: u32,
}

pub struct VerticalScrollView<'s, 'l, V> {
    pub view: V,
    pub scroll_bar_style: &'s VerticalScrollBarStyle,
    pub limits: &'l mut VerticalScrollLimits,
    pub state: VerticalScrollState,
}

impl VerticalScrollBarStyle {
    pub fn new() -> Self {
        Self {
            style: Style::new(),
            character: '█',
            left_padding: 1,
        }
    }
}

impl VerticalScrollLimits {
    pub fn new() -> Self {
        Self {
            last_rendered_inner_height: 0,
            last_rendered_outer_height: 0,
        }
    }
    pub fn max_scroll_position(&self) -> u32 {
        self.last_rendered_inner_height
            .saturating_sub(self.last_rendered_outer_height)
    }
}

impl VerticalScrollState {
    pub fn new() -> Self {
        Self { scroll_position: 0 }
    }
    pub fn scroll_to(&mut self, scroll_position: u32, limits: &VerticalScrollLimits) {
        self.scroll_position = scroll_position.min(limits.max_scroll_position());
    }
    pub fn scroll_up_lines(&mut self, num_lines: u32, limits: &VerticalScrollLimits) {
        let _ = limits;
        self.scroll_position = self.scroll_position.saturating_sub(num_lines);
    }
    pub fn scroll_down_lines(&mut self, num_lines: u32, limits: &VerticalScrollLimits) {
        let scroll_position = self.scroll_position;
        self.scroll_to(scroll_position + num_lines, limits)
    }
    pub fn scroll_lines(&mut self, num_lines: i32, limits: &VerticalScrollLimits) {
        if num_lines < 0 {
            self.scroll_up_lines((-num_lines) as u32, limits);
        } else {
            self.scroll_down_lines(num_lines as u32, limits);
        }
    }
    pub fn scroll_up_line(&mut self, limits: &VerticalScrollLimits) {
        self.scroll_up_lines(1, limits);
    }
    pub fn scroll_down_line(&mut self, limits: &VerticalScrollLimits) {
        self.scroll_down_lines(1, limits);
    }
    pub fn scroll_up_page(&mut self, limits: &VerticalScrollLimits) {
        self.scroll_up_lines(limits.last_rendered_outer_height as u32, limits);
    }
    pub fn scroll_down_page(&mut self, limits: &VerticalScrollLimits) {
        self.scroll_down_lines(limits.last_rendered_outer_height as u32, limits);
    }
    pub fn scroll_to_top(&mut self, limits: &VerticalScrollLimits) {
        let _ = limits;
        self.scroll_position = 0;
    }
    pub fn scroll_to_bottom(&mut self, limits: &VerticalScrollLimits) {
        self.scroll_position = limits.max_scroll_position();
    }
    pub fn scroll_position(&self) -> u32 {
        self.scroll_position
    }
}

fn render_scroll_bar<F: Frame, C: ColModify>(
    scroll_bar_style: &VerticalScrollBarStyle,
    state: VerticalScrollState,
    limits: VerticalScrollLimits,
    context: ViewContext<C>,
    frame: &mut F,
) {
    if limits.last_rendered_inner_height > limits.last_rendered_outer_height {
        let view_cell = ViewCell {
            style: scroll_bar_style.style,
            character: Some(scroll_bar_style.character),
        };
        let bar_x = context.size.width() as i32 - 1;
        let bar_height =
            (limits.last_rendered_outer_height * limits.last_rendered_outer_height) / limits.last_rendered_inner_height;
        let bar_top = ((limits.last_rendered_outer_height - bar_height) * state.scroll_position as u32)
            / limits.max_scroll_position() as u32;
        for y in 0..bar_height {
            let bar_y = (y + bar_top) as i32;
            let coord = Coord::new(bar_x, bar_y);
            frame.set_cell_relative(coord, 0, view_cell, context);
        }
    }
}

struct PartialFrame<'a, F> {
    offset: Coord,
    max_y: i32,
    frame: &'a mut F,
}

impl<'a, F> Frame for PartialFrame<'a, F>
where
    F: Frame,
{
    fn set_cell_relative<C: ColModify>(
        &mut self,
        relative_coord: Coord,
        relative_depth: i8,
        relative_cell: ViewCell,
        context: ViewContext<C>,
    ) {
        let adjusted_relative_coord = relative_coord - self.offset;
        self.max_y = self.max_y.max((relative_coord + context.offset).y);
        if adjusted_relative_coord.is_valid(context.size) {
            let absolute_coord = adjusted_relative_coord + context.offset;
            let absolute_depth = relative_depth + context.depth;
            let absolute_cell = ViewCell {
                style: Style {
                    foreground: context.col_modify.foreground(relative_cell.style.foreground),
                    background: context.col_modify.background(relative_cell.style.background),
                    ..relative_cell.style
                },
                ..relative_cell
            };
            self.set_cell_absolute(absolute_coord, absolute_depth, absolute_cell);
        }
    }

    fn set_cell_absolute(&mut self, absolute_coord: Coord, absolute_depth: i8, absolute_cell: ViewCell) {
        self.frame
            .set_cell_absolute(absolute_coord, absolute_depth, absolute_cell);
    }
}

impl<'s, 'l, V, T> View<T> for VerticalScrollView<'s, 'l, V>
where
    V: View<T>,
{
    fn view<F: Frame, C: ColModify>(&mut self, data: T, context: ViewContext<C>, frame: &mut F) {
        let mut partial_frame = PartialFrame {
            offset: Coord::new(0, self.state.scroll_position as i32),
            max_y: 0,
            frame,
        };
        self.view.view(
            data,
            context.constrain_size_by(Size::new(1 + self.scroll_bar_style.left_padding, 0)),
            &mut partial_frame,
        );
        self.limits.last_rendered_inner_height = (partial_frame.max_y - context.offset.y).max(0) as u32 + 1;
        self.limits.last_rendered_outer_height = context.size.height();
        render_scroll_bar(self.scroll_bar_style, self.state, *self.limits, context, frame);
    }
}
