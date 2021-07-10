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

    fn process(&self, mut cursor: Coord, ctx: Ctx, fb: &mut FrameBuffer) -> Coord {
        for character in self.string.chars() {
            cursor = Self::process_character(cursor, character, self.style, ctx, fb);
        }
        cursor
    }

    pub fn wrap_char(self) -> StyledStringCharWrapped {
        StyledStringCharWrapped {
            styled_string: self,
        }
    }

    pub fn wrap_word(self) -> StyledStringWordWrapped {
        StyledStringWordWrapped {
            styled_string: self,
            state: RefCell::new(WordWrapState::default()),
        }
    }
}

impl PureStaticComponent for StyledString {
    fn render(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        self.process(Coord::new(0, 0), ctx, fb);
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

    fn process_styled_string(
        styled_string: &StyledString,
        mut cursor: Coord,
        ctx: Ctx,
        fb: &mut FrameBuffer,
    ) -> Coord {
        for character in styled_string.string.chars() {
            cursor = Self::process_character(cursor, character, styled_string.style, ctx, fb);
        }
        cursor
    }
}

impl PureStaticComponent for StyledStringCharWrapped {
    fn render(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        Self::process_styled_string(&self.styled_string, Coord::new(0, 0), ctx, fb);
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
        state.process_styled_string(&self.styled_string, ctx, fb);
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

    fn process_character(&mut self, character: char, style: Style, ctx: Ctx, fb: &mut FrameBuffer) {
        if ctx.bounding_box.size().width() == 0 {
            return;
        }
        match character {
            '\n' => {
                self.flush(ctx, fb);
                self.cursor.x = 0;
                self.cursor.y += 1;
            }
            '\r' => {
                self.flush(ctx, fb);
                self.cursor.x = 0;
            }
            ' ' => {
                self.flush(ctx, fb);
                if self.cursor.x != 0 {
                    let render_cell = RenderCell {
                        character: Some(' '),
                        style,
                    };
                    fb.set_cell_relative_to_ctx(ctx, self.cursor, 0, render_cell);
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
                        self.flush(ctx, fb);
                    } else {
                        self.cursor.x = 0;
                        self.cursor.y += 1;
                    }
                }
            }
        }
    }

    fn flush(&mut self, ctx: Ctx, fb: &mut FrameBuffer) {
        if ctx.bounding_box.size().width() == 0 {
            self.current_word_buffer.clear();
            return;
        }
        for render_cell in self.current_word_buffer.drain(..) {
            fb.set_cell_relative_to_ctx(ctx, self.cursor, 0, render_cell);
            self.cursor.x += 1;
        }
        assert!(self.cursor.x as u32 <= ctx.bounding_box.size().width());
        if self.cursor.x as u32 == ctx.bounding_box.size().width() {
            self.cursor.x = 0;
            self.cursor.y += 1;
        }
    }

    fn process_styled_string(
        &mut self,
        styled_string: &StyledString,
        ctx: Ctx,
        fb: &mut FrameBuffer,
    ) {
        for character in styled_string.string.chars() {
            self.process_character(character, styled_string.style, ctx, fb);
        }
    }
}

pub struct Text {
    pub parts: Vec<StyledString>,
}

impl Text {
    pub fn new(parts: Vec<StyledString>) -> Self {
        Self { parts }
    }

    pub fn wrap_char(self) -> TextCharWrapped {
        TextCharWrapped { text: self }
    }

    pub fn wrap_word(self) -> TextWordWrapped {
        TextWordWrapped {
            text: self,
            state: RefCell::new(WordWrapState::default()),
        }
    }
}

impl PureStaticComponent for Text {
    fn render(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        let mut cursor = Coord::new(0, 0);
        for part in self.parts.iter() {
            cursor = part.process(cursor, ctx, fb);
        }
    }
}

pub struct TextCharWrapped {
    pub text: Text,
}

impl PureStaticComponent for TextCharWrapped {
    fn render(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        let mut cursor = Coord::new(0, 0);
        for part in self.text.parts.iter() {
            cursor = StyledStringCharWrapped::process_styled_string(part, cursor, ctx, fb);
        }
    }
}

pub struct TextWordWrapped {
    pub text: Text,
    state: RefCell<WordWrapState>,
}
impl PureStaticComponent for TextWordWrapped {
    fn render(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        let mut state = self.state.borrow_mut();
        state.clear();
        for part in self.text.parts.iter() {
            state.process_styled_string(part, ctx, fb);
        }
        state.flush(ctx, fb);
    }
}

impl From<StyledString> for Text {
    fn from(styled_string: StyledString) -> Self {
        Text::new(vec![styled_string])
    }
}

impl From<Vec<StyledString>> for Text {
    fn from(parts: Vec<StyledString>) -> Self {
        Text::new(parts)
    }
}
