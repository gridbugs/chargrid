use prototty_render::*;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

pub trait Wrap: private_wrap::Sealed {
    #[doc(hidden)]
    fn clear(&mut self);
    #[doc(hidden)]
    fn process_character<F: Frame, C: ColModify>(
        &mut self,
        character: char,
        style: Style,
        context: ViewContext<C>,
        frame: &mut F,
    );
    #[doc(hidden)]
    fn flush<F: Frame, C: ColModify>(&mut self, context: ViewContext<C>, frame: &mut F) {
        let _ = context;
        let _ = frame;
    }
    #[doc(hidden)]
    fn num_lines(&self) -> usize;
}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct None {
    cursor: Coord,
}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Word {
    cursor: Coord,
    current_word_buffer: Vec<ViewCell>,
}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
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
    fn process_character<F: Frame, C: ColModify>(
        &mut self,
        character: char,
        style: Style,
        context: ViewContext<C>,
        frame: &mut F,
    ) {
        match character {
            '\n' => {
                self.cursor.x = 0;
                self.cursor.y += 1;
            }
            '\r' => self.cursor.x = 0,
            other => {
                let view_cell = ViewCell {
                    character: Some(other),
                    style,
                };
                frame.set_cell_relative(self.cursor, 0, view_cell, context);
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

    fn process_character<F: Frame, C: ColModify>(
        &mut self,
        character: char,
        style: Style,
        context: ViewContext<C>,
        frame: &mut F,
    ) {
        match character {
            '\n' => {
                self.flush(context, frame);
                self.cursor.x = 0;
                self.cursor.y += 1;
            }
            '\r' => {
                self.flush(context, frame);
                self.cursor.x = 0;
            }
            ' ' => {
                self.flush(context, frame);
                if self.cursor.x != 0 {
                    let view_cell = ViewCell {
                        character: Some(' '),
                        style,
                    };
                    frame.set_cell_relative(self.cursor, 0, view_cell, context);
                    self.cursor.x += 1;
                    //assert!(self.cursor.x as u32 <= context.size.width());
                    if self.cursor.x as u32 >= context.size.width() {
                        self.cursor.x = 0;
                        self.cursor.y += 1;
                    }
                }
            }
            other => {
                let view_cell = ViewCell {
                    character: Some(other),
                    style,
                };
                self.current_word_buffer.push(view_cell);
                //assert!(self.cursor.x as u32 + self.current_word_buffer.len() as u32 <= context.size.width());
                if self.cursor.x as u32 + self.current_word_buffer.len() as u32 >= context.size.width() {
                    if self.cursor.x == 0 {
                        self.flush(context, frame);
                    } else {
                        self.cursor.x = 0;
                        self.cursor.y += 1;
                    }
                }
            }
        }
    }

    fn flush<F: Frame, C: ColModify>(&mut self, context: ViewContext<C>, frame: &mut F) {
        for view_cell in self.current_word_buffer.drain(..) {
            frame.set_cell_relative(self.cursor, 0, view_cell, context);
            self.cursor.x += 1;
        }
        //assert!(self.cursor.x as u32 <= context.size.width());
        if self.cursor.x as u32 >= context.size.width() {
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

    fn process_character<F: Frame, C: ColModify>(
        &mut self,
        character: char,
        style: Style,
        context: ViewContext<C>,
        frame: &mut F,
    ) {
        match character {
            '\n' => {
                self.cursor.x = 0;
                self.cursor.y += 1;
            }
            '\r' => self.cursor.x = 0,
            other => {
                let view_cell = ViewCell {
                    character: Some(other),
                    style,
                };
                frame.set_cell_relative(self.cursor, 0, view_cell, context);
                self.cursor += Coord::new(1, 0);
                if self.cursor.x >= context.size.width() as i32 {
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
