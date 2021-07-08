use chargrid_component::*;
use std::cell::RefCell;

pub struct StyledString {
    pub string: String,
    pub style: Style,
}

impl StyledString {
    fn process_character(
        mut cursor: Coord,
        character: char,
        style: Style,
        ctx: Ctx,
        frame: &mut FrameBuffer,
    ) -> Coord {
        match character {
            '\n' => {
                cursor.x = 0;
                cursor.y += 1;
            }
            '\r' => cursor.x = 0,
            other => {
                let render_cell = RenderCell {
                    character: Some(other),
                    style,
                };
                frame.set_cell_relative_to_ctx(ctx, cursor, 0, render_cell);
                cursor += Coord::new(1, 0);
            }
        }
        cursor
    }

    pub fn wrap_word(self) -> StyledStringWordWrapped {
        StyledStringWordWrapped {
            styled_string: self,
            state: RefCell::new(WordWrapState::default()),
        }
    }

    pub fn wrap_char(self) -> StyledStringCharWrapped {
        StyledStringCharWrapped {
            styled_string: self,
        }
    }
}

impl PureStaticComponent for StyledString {
    fn render(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        let mut cursor = Coord::new(0, 0);
        for character in self.string.chars() {
            cursor = Self::process_character(cursor, character, self.style, ctx, fb);
        }
    }
}

pub struct StyledStringCharWrapped {
    pub styled_string: StyledString,
}

impl StyledStringCharWrapped {
    fn process_character(
        mut cursor: Coord,
        character: char,
        style: Style,
        ctx: Ctx,
        frame: &mut FrameBuffer,
    ) -> Coord {
        match character {
            '\n' => {
                cursor.x = 0;
                cursor.y += 1;
            }
            '\r' => cursor.x = 0,
            other => {
                let render_cell = RenderCell {
                    character: Some(other),
                    style,
                };
                frame.set_cell_relative_to_ctx(ctx, cursor, 0, render_cell);
                cursor += Coord::new(1, 0);
                if cursor.x >= ctx.bounding_box.size().width() as i32 {
                    cursor.x = 0;
                    cursor.y += 1;
                }
            }
        }
        cursor
    }
}

impl PureStaticComponent for StyledStringCharWrapped {
    fn render(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        let mut cursor = Coord::new(0, 0);
        for character in self.styled_string.string.chars() {
            cursor = Self::process_character(cursor, character, self.styled_string.style, ctx, fb);
        }
    }
}

pub struct StyledStringWordWrapped {
    pub styled_string: StyledString,
    state: RefCell<WordWrapState>,
}

impl PureStaticComponent for StyledStringWordWrapped {
    fn render(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        let mut state = self.state.borrow_mut();
        state.clear();
        for character in self.styled_string.string.chars() {
            state.process_character(character, self.styled_string.style, ctx, fb);
        }
        state.flush(ctx, fb);
    }
}

#[derive(Default)]
struct WordWrapState {
    cursor: Coord,
    current_word_buffer: Vec<RenderCell>,
}

impl WordWrapState {
    fn clear(&mut self) {
        self.cursor = Coord::new(0, 0);
        self.current_word_buffer.clear();
    }

    fn process_character(
        &mut self,
        character: char,
        style: Style,
        ctx: Ctx,
        frame: &mut FrameBuffer,
    ) {
        if ctx.bounding_box.size().width() == 0 {
            return;
        }
        match character {
            '\n' => {
                self.flush(ctx, frame);
                self.cursor.x = 0;
                self.cursor.y += 1;
            }
            '\r' => {
                self.flush(ctx, frame);
                self.cursor.x = 0;
            }
            ' ' => {
                self.flush(ctx, frame);
                if self.cursor.x != 0 {
                    let render_cell = RenderCell {
                        character: Some(' '),
                        style,
                    };
                    frame.set_cell_relative_to_ctx(ctx, self.cursor, 0, render_cell);
                    self.cursor.x += 1;
                    assert!(self.cursor.x as u32 <= ctx.bounding_box.size().width());
                    if self.cursor.x as u32 == ctx.bounding_box.size().width() {
                        self.cursor.x = 0;
                        self.cursor.y += 1;
                    }
                }
            }
            other => {
                let render_cell = RenderCell {
                    character: Some(other),
                    style,
                };
                self.current_word_buffer.push(render_cell);
                assert!(
                    self.cursor.x as u32 + self.current_word_buffer.len() as u32
                        <= ctx.bounding_box.size().width()
                );
                if self.cursor.x as u32 + self.current_word_buffer.len() as u32
                    == ctx.bounding_box.size().width()
                {
                    if self.cursor.x == 0 {
                        self.flush(ctx, frame);
                    } else {
                        self.cursor.x = 0;
                        self.cursor.y += 1;
                    }
                }
            }
        }
    }

    fn flush(&mut self, ctx: Ctx, frame: &mut FrameBuffer) {
        if ctx.bounding_box.size().width() == 0 {
            self.current_word_buffer.clear();
            return;
        }
        for render_cell in self.current_word_buffer.drain(..) {
            frame.set_cell_relative_to_ctx(ctx, self.cursor, 0, render_cell);
            self.cursor.x += 1;
        }
        assert!(self.cursor.x as u32 <= ctx.bounding_box.size().width());
        if self.cursor.x as u32 == ctx.bounding_box.size().width() {
            self.cursor.x = 0;
            self.cursor.y += 1;
        }
    }
}
