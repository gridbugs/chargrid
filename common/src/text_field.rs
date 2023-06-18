use chargrid_core::*;

pub struct TextField {
    text: Vec<char>,
    width: u32,
    cursor_position: usize,
    cursor_rgba32: Rgba32,
    text_style: Style,
}

impl TextField {
    pub fn with_initial_string(width: u32, string: String) -> Self {
        let text = string.chars().collect::<Vec<_>>();
        Self {
            width,
            cursor_position: text.len(),
            cursor_rgba32: Rgba32::new_grey(63),
            text_style: Style::plain_text(),
            text,
        }
    }

    fn add_character(&mut self, ch: char) {
        assert!(self.cursor_position <= self.text.len());
        if self.cursor_position == self.text.len() {
            self.text.push(ch);
        } else {
            self.text.insert(self.cursor_position, ch);
        }
        self.cursor_position += 1;
    }

    fn backspace(&mut self) {
        assert!(self.cursor_position <= self.text.len());
        if self.cursor_position > 0 {
            self.text.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }

    fn delete(&mut self) {
        assert!(self.cursor_position <= self.text.len());
        if self.cursor_position < self.text.len() {
            self.text.remove(self.cursor_position);
        }
    }

    fn left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    fn right(&mut self) {
        if self.cursor_position < self.text.len() {
            self.cursor_position += 1;
        }
    }
}

impl Component for TextField {
    type Output = Option<String>;
    type State = ();

    fn render(&self, _state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        for (i, &ch) in self.text.iter().enumerate().take(self.width as usize) {
            let render_cell = RenderCell {
                character: Some(ch),
                style: self.text_style,
            };
            fb.set_cell_relative_to_ctx(ctx, Coord::new(i as i32, 0), 0, render_cell);
        }
        let cursor_render_cell = RenderCell {
            character: None,
            style: Style::default().with_background(self.cursor_rgba32),
        };
        fb.set_cell_relative_to_ctx(
            ctx,
            Coord::new(self.cursor_position as i32, 0),
            0,
            cursor_render_cell,
        );
    }

    fn update(&mut self, _state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        if let Some(input) = event.input() {
            use input::*;
            match input {
                Input::Mouse(
                    MouseInput::MousePress { coord, .. }
                    | MouseInput::MouseMove {
                        coord,
                        button: Some(_),
                    },
                ) => {
                    if let Some(coord) = ctx.bounding_box.coord_absolute_to_relative(coord) {
                        if coord.x as usize <= self.text.len() {
                            self.cursor_position = coord.x as usize;
                        }
                    }
                }
                Input::Keyboard(KeyboardInput {
                    key,
                    event: KeyboardEvent::KeyPress,
                }) => match key {
                    keys::RETURN => return Some(self.text.iter().collect::<String>()),
                    Key::Left => self.left(),
                    Key::Right => self.right(),
                    Key::Delete => self.delete(),
                    keys::BACKSPACE => self.backspace(),
                    Key::Char(ch) => {
                        if !ch.is_control() {
                            self.add_character(ch);
                        }
                    }
                    _ => (),
                },
                _ => (),
            }
        }
        None
    }

    fn size(&self, _state: &Self::State, _ctx: Ctx) -> Size {
        Size::new(self.width, 1)
    }
}
