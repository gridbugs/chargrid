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
}

pub struct VerticalScrolled<V> {
    pub view: V,
    pub scrollbar: VerticalScrollbar,
    state: VerticalScrollState,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct VerticalScrollState {
    last_rendered_inner_height: u32,
    last_rendered_outer_height: u32,
    scroll_position: usize,
}

impl VerticalScrollState {
    pub fn new() -> Self {
        Self {
            last_rendered_inner_height: 0,
            last_rendered_outer_height: 0,
            scroll_position: 0,
        }
    }
    pub fn scroll_to(&mut self, scroll_position: usize) {
        self.scroll_position = scroll_position.min(self.max_scroll_position());
    }
    pub fn scroll_up_lines(&mut self, num_lines: usize) {
        self.scroll_position = self.scroll_position.saturating_sub(num_lines);
    }
    pub fn scroll_down_lines(&mut self, num_lines: usize) {
        let scroll_position = self.scroll_position;
        self.scroll_to(scroll_position + num_lines)
    }
    pub fn scroll_lines(&mut self, num_lines: isize) {
        if num_lines < 0 {
            self.scroll_up_lines((-num_lines) as usize);
        } else {
            self.scroll_down_lines(num_lines as usize);
        }
    }
    pub fn scroll_up_line(&mut self) {
        self.scroll_up_lines(1);
    }
    pub fn scroll_down_line(&mut self) {
        self.scroll_down_lines(1);
    }
    pub fn scroll_up_page(&mut self) {
        let height = self.last_rendered_outer_height as usize;
        self.scroll_up_lines(height);
    }
    pub fn scroll_down_page(&mut self) {
        let height = self.last_rendered_outer_height as usize;
        self.scroll_down_lines(height);
    }
    pub fn scroll_to_top(&mut self) {
        self.scroll_position = 0;
    }
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_position = self.max_scroll_position();
    }
    pub fn max_scroll_position(&self) -> usize {
        self.last_rendered_inner_height
            .saturating_sub(self.last_rendered_outer_height) as usize
    }
    pub fn scroll_position(&self) -> usize {
        self.scroll_position
    }
}

impl<V> VerticalScrolled<V> {
    pub fn new(view: V, scrollbar: VerticalScrollbar) -> Self {
        Self {
            view,
            scrollbar,
            state: VerticalScrollState::new(),
        }
    }
    pub fn state(&self) -> VerticalScrollState {
        self.state
    }
    pub fn sync_scroll_position(&mut self, state: &VerticalScrollState) {
        self.state.scroll_position = state.scroll_position;
    }
    fn view_scrollbar<G: ViewGrid, R: ViewTransformRgb24>(
        &self,
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        if self.state.last_rendered_inner_height > self.state.last_rendered_outer_height {
            let view_cell = ViewCell {
                style: self.scrollbar.style,
                character: Some(self.scrollbar.character),
            };
            let bar_x = context.size.width() as i32 - 1;
            let bar_height = (self.state.last_rendered_outer_height
                * self.state.last_rendered_outer_height)
                / self.state.last_rendered_inner_height;
            let bar_top = ((self.state.last_rendered_outer_height - bar_height)
                * self.state.scroll_position as u32)
                / self.state.max_scroll_position() as u32;
            for y in 0..bar_height {
                let bar_y = (y + bar_top) as i32;
                let coord = Coord::new(bar_x, bar_y);
                grid.set_cell_relative(coord, 0, view_cell, context);
            }
        }
    }
}

impl<T: Clone, V: View<T>> View<T> for VerticalScrolled<V> {
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        data: T,
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        let inner_size = self.view.view_reporting_intended_size(
            data,
            context
                .constrain_size_by(Size::new(1 + self.scrollbar.padding, 0))
                .add_inner_offset(Coord::new(0, -(self.state.scroll_position as i32))),
            grid,
        );
        self.state.last_rendered_inner_height = inner_size.height();
        self.state.last_rendered_outer_height = context.size.height();
        self.view_scrollbar(context, grid);
    }
}
