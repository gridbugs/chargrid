use chargrid_component::*;

pub struct StyledString {
    pub string: String,
    pub style: Style,
}

impl PureComponent for StyledString {
    type PureOutput = ();

    fn pure_render(&self, ctx: Ctx, fb: &mut FrameBuffer) {}
    fn pure_update(&mut self, _ctx: Ctx, event: Event) -> Self::PureOutput {}
}

pub mod wrap {
    use chargrid_component::*;

    pub trait Wrap: private_wrap::Sealed {
        #[doc(hidden)]
        fn clear(&mut self);
        #[doc(hidden)]
        fn process_character(
            &mut self,
            character: char,
            style: Style,
            ctx: Ctx,
            frame: &mut FrameBuffer,
        );
        #[doc(hidden)]
        fn flush(&mut self, ctx: Ctx, frame: &mut FrameBuffer) {
            let _ = ctx;
            let _ = frame;
        }
        #[doc(hidden)]
        fn num_lines(&self) -> usize;
    }

    #[derive(Debug, Clone)]
    pub struct None {
        cursor: Coord,
    }

    #[derive(Debug, Clone)]
    pub struct Word {
        cursor: Coord,
        current_word_buffer: Vec<RenderCell>,
    }

    #[derive(Debug, Clone)]
    pub struct Char {
        cursor: Coord,
    }

    impl None {
        pub fn new() -> Self {
            Self {
                cursor: Coord::new(0, 0),
            }
        }
    }

    impl Word {
        pub fn new() -> Self {
            Self {
                cursor: Coord::new(0, 0),
                current_word_buffer: Vec::new(),
            }
        }
    }

    impl Char {
        pub fn new() -> Self {
            Self {
                cursor: Coord::new(0, 0),
            }
        }
    }

    impl Default for None {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Default for Word {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Default for Char {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Wrap for None {
        fn clear(&mut self) {
            self.cursor = Coord::new(0, 0);
        }
        fn process_character(
            &mut self,
            character: char,
            style: Style,
            ctx: Ctx,
            frame: &mut FrameBuffer,
        ) {
            match character {
                '\n' => {
                    self.cursor.x = 0;
                    self.cursor.y += 1;
                }
                '\r' => self.cursor.x = 0,
                other => {
                    let render_cell = RenderCell {
                        character: Some(other),
                        style,
                    };
                    frame.set_cell_relative_to_ctx(ctx, self.cursor, 0, render_cell);
                    self.cursor += Coord::new(1, 0);
                }
            }
        }
        fn num_lines(&self) -> usize {
            self.cursor.y as usize + 1
        }
    }

    impl Wrap for Word {
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
            if ctx.bounding_box.size.width() == 0 {
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
                        assert!(self.cursor.x as u32 <= ctx.bounding_box.size.width());
                        if self.cursor.x as u32 == ctx.bounding_box.size.width() {
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
                            <= ctx.bounding_box.size.width()
                    );
                    if self.cursor.x as u32 + self.current_word_buffer.len() as u32
                        == ctx.bounding_box.size.width()
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
            if ctx.bounding_box.size.width() == 0 {
                self.current_word_buffer.clear();
                return;
            }
            for render_cell in self.current_word_buffer.drain(..) {
                frame.set_cell_relative_to_ctx(ctx, self.cursor, 0, render_cell);
                self.cursor.x += 1;
            }
            assert!(self.cursor.x as u32 <= ctx.bounding_box.size.width());
            if self.cursor.x as u32 == ctx.bounding_box.size.width() {
                self.cursor.x = 0;
                self.cursor.y += 1;
            }
        }

        fn num_lines(&self) -> usize {
            self.cursor.y as usize + 1
        }
    }

    impl Wrap for Char {
        fn clear(&mut self) {
            self.cursor = Coord::new(0, 0);
        }

        fn process_character(
            &mut self,
            character: char,
            style: Style,
            ctx: Ctx,
            frame: &mut FrameBuffer,
        ) {
            match character {
                '\n' => {
                    self.cursor.x = 0;
                    self.cursor.y += 1;
                }
                '\r' => self.cursor.x = 0,
                other => {
                    let render_cell = RenderCell {
                        character: Some(other),
                        style,
                    };
                    frame.set_cell_relative_to_ctx(ctx, self.cursor, 0, render_cell);
                    self.cursor += Coord::new(1, 0);
                    if self.cursor.x >= ctx.bounding_box.size.width() as i32 {
                        self.cursor.x = 0;
                        self.cursor.y += 1;
                    }
                }
            }
        }

        fn num_lines(&self) -> usize {
            self.cursor.y as usize + 1
        }
    }

    mod private_wrap {
        pub trait Sealed {}
        impl Sealed for super::None {}
        impl Sealed for super::Word {}
        impl Sealed for super::Char {}
    }
}
