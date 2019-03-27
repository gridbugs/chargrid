use prototty_render::*;

pub struct VerticalScrollbar {
    pub style: Style,
    pub character: char,
    pub padding: u32,
}

impl Default for VerticalScrollbar {
    fn default() -> Self {
        Self {
            style: Default::default(),
            character: '█',
            padding: 1,
        }
    }
}

impl VerticalScrollbar {
    pub fn new(style: Style, character: char, padding: u32) -> Self {
        Self {
            style,
            character,
            padding,
        }
    }
    pub fn padding(&self) -> u32 {
        self.padding
    }
    fn view<V, G: ViewGrid, R: ViewTransformRgb24>(
        &self,
        state: &VerticalScrollState,
        scroll: &VerticalScroll<V>,
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        if scroll.last_rendered_inner_height > scroll.last_rendered_outer_height {
            let view_cell = ViewCell {
                style: self.style,
                character: Some(self.character),
            };
            let bar_x = context.size.width() as i32 - 1;
            let bar_height = (scroll.last_rendered_outer_height
                * scroll.last_rendered_outer_height)
                / scroll.last_rendered_inner_height;
            let bar_top = ((scroll.last_rendered_outer_height - bar_height)
                * state.scroll_position as u32)
                / scroll.max_scroll_position() as u32;
            for y in 0..bar_height {
                let bar_y = (y + bar_top) as i32;
                let coord = Coord::new(bar_x, bar_y);
                grid.set_cell_relative(coord, 0, view_cell, context);
            }
        }
    }
}

pub struct VerticalScroll<V> {
    pub view: V,
    last_rendered_inner_height: u32,
    last_rendered_outer_height: u32,
}

impl<V> VerticalScroll<V> {
    pub fn new(view: V) -> Self {
        Self {
            view,
            last_rendered_inner_height: 0,
            last_rendered_outer_height: 0,
        }
    }
    pub fn max_scroll_position(&self) -> usize {
        self.last_rendered_inner_height
            .saturating_sub(self.last_rendered_outer_height) as usize
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct VerticalScrollState {
    scroll_position: usize,
}

impl VerticalScrollState {
    pub fn new() -> Self {
        Self { scroll_position: 0 }
    }
    pub fn scroll_to<V>(&mut self, scroll_position: usize, scroll: &VerticalScroll<V>) {
        self.scroll_position = scroll_position.min(scroll.max_scroll_position());
    }
    pub fn scroll_up_lines<V>(&mut self, num_lines: usize, scroll: &VerticalScroll<V>) {
        let _ = scroll;
        self.scroll_position = self.scroll_position.saturating_sub(num_lines);
    }
    pub fn scroll_down_lines<V>(&mut self, num_lines: usize, scroll: &VerticalScroll<V>) {
        let scroll_position = self.scroll_position;
        self.scroll_to(scroll_position + num_lines, scroll)
    }
    pub fn scroll_lines<V>(&mut self, num_lines: isize, scroll: &VerticalScroll<V>) {
        if num_lines < 0 {
            self.scroll_up_lines((-num_lines) as usize, scroll);
        } else {
            self.scroll_down_lines(num_lines as usize, scroll);
        }
    }
    pub fn scroll_up_line<V>(&mut self, scroll: &VerticalScroll<V>) {
        self.scroll_up_lines(1, scroll);
    }
    pub fn scroll_down_line<V>(&mut self, scroll: &VerticalScroll<V>) {
        self.scroll_down_lines(1, scroll);
    }
    pub fn scroll_up_page<V>(&mut self, scroll: &VerticalScroll<V>) {
        self.scroll_up_lines(scroll.last_rendered_outer_height as usize, scroll);
    }
    pub fn scroll_down_page<V>(&mut self, scroll: &VerticalScroll<V>) {
        self.scroll_down_lines(scroll.last_rendered_outer_height as usize, scroll);
    }
    pub fn scroll_to_top<V>(&mut self, scroll: &VerticalScroll<V>) {
        let _ = scroll;
        self.scroll_position = 0;
    }
    pub fn scroll_to_bottom<V>(&mut self, scroll: &VerticalScroll<V>) {
        self.scroll_position = scroll.max_scroll_position();
    }
    pub fn scroll_position(&self) -> usize {
        self.scroll_position
    }
}

impl<'a, T: Clone, V: View<T>> View<(T, &'a VerticalScrollState, &'a VerticalScrollbar)>
    for VerticalScroll<V>
{
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        (data, state, scrollbar): (T, &'a VerticalScrollState, &'a VerticalScrollbar),
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        let inner_size = self.view.view_reporting_intended_size(
            data,
            context
                .constrain_size_by(Size::new(1 + scrollbar.padding, 0))
                .add_inner_offset(Coord::new(0, -(state.scroll_position as i32))),
            grid,
        );
        self.last_rendered_inner_height = inner_size.height();
        self.last_rendered_outer_height = context.size.height();
        scrollbar.view(state, self, context, grid);
    }
}

impl<'a, T: Clone, V: View<T>> View<(T, &'a VerticalScrollState)> for VerticalScroll<V> {
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        (data, state): (T, &'a VerticalScrollState),
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        let inner_size = self.view.view_reporting_intended_size(
            data,
            context.add_inner_offset(Coord::new(0, -(state.scroll_position as i32))),
            grid,
        );
        self.last_rendered_inner_height = inner_size.height();
        self.last_rendered_outer_height = context.size.height();
    }
}
