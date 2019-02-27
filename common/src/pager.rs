use prototty_input::*;
use prototty_render::*;
use std::mem;
use text_info::*;

struct Buffer {
    current_word: String,
    wrapped_lines: Vec<String>,
    num_lines: usize,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            current_word: String::new(),
            wrapped_lines: vec![String::new()],
            num_lines: 0,
        }
    }

    fn emit_word(&mut self, mut index_of_current_line: usize, width: usize) -> usize {
        if index_of_current_line == self.wrapped_lines.len() {
            assert!(self.current_word.len() <= width);
            self.wrapped_lines
                .push(mem::replace(&mut self.current_word, String::new()));
        } else {
            if self.wrapped_lines[index_of_current_line].len() + self.current_word.len() <= width {
                self.wrapped_lines[index_of_current_line].push_str(&self.current_word);
            } else {
                index_of_current_line += 1;
                if index_of_current_line == self.wrapped_lines.len() {
                    self.wrapped_lines
                        .push(mem::replace(&mut self.current_word, String::new()));
                } else {
                    mem::swap(
                        &mut self.current_word,
                        &mut self.wrapped_lines[index_of_current_line],
                    );
                }
            }
            self.current_word.clear();
        }
        index_of_current_line
    }

    fn emit_space(&mut self, mut index_of_current_line: usize, width: usize) -> usize {
        assert!(self.current_word.is_empty());
        if index_of_current_line == self.wrapped_lines.len() {
            self.wrapped_lines.push(String::new());
        }
        let current_line_len = self.wrapped_lines[index_of_current_line].len();
        if current_line_len > 0 && current_line_len < width {
            self.wrapped_lines[index_of_current_line].push(' ');
        } else if current_line_len == width {
            index_of_current_line += 1;
            if index_of_current_line == self.wrapped_lines.len() {
                self.wrapped_lines.push(String::new());
            } else {
                self.wrapped_lines[index_of_current_line].clear();
            }
        } else {
            assert_eq!(current_line_len, 0);
        }
        index_of_current_line
    }

    fn emit_newline(&mut self, mut index_of_current_line: usize) -> usize {
        index_of_current_line += 1;
        if index_of_current_line == self.wrapped_lines.len() {
            self.wrapped_lines.push(String::new());
        } else {
            self.wrapped_lines[index_of_current_line].clear();
        }
        index_of_current_line
    }

    fn update(&mut self, string: &str, width: usize) {
        self.current_word.clear();
        self.wrapped_lines[0].clear();
        let mut index_of_current_line = 0;
        for ch in string.chars() {
            if ch == ' ' {
                if self.current_word.is_empty() {
                    index_of_current_line = self.emit_space(index_of_current_line, width);
                } else {
                    index_of_current_line = self.emit_word(index_of_current_line, width);
                    index_of_current_line = self.emit_space(index_of_current_line, width);
                }
            } else if ch == '\n' {
                if !self.current_word.is_empty() {
                    index_of_current_line = self.emit_word(index_of_current_line, width);
                }
                index_of_current_line = self.emit_newline(index_of_current_line);
            } else {
                if self.current_word.len() == width {
                    index_of_current_line = self.emit_word(index_of_current_line, width);
                }
                self.current_word.push(ch);
            }
        }
        if !self.current_word.is_empty() {
            index_of_current_line = self.emit_word(index_of_current_line, width);
        }
        self.num_lines = index_of_current_line + 1;
    }

    fn into_wrapped_lines(self) -> Vec<String> {
        self.wrapped_lines
    }
}

pub struct Pager {
    size: Size,
    text_info: TextInfo,
    scroll_position: usize,
    wrapped_lines: Vec<String>,
}

impl Pager {
    pub fn new<T: AsRef<str>>(string: T, size: Size, text_info: TextInfo) -> Self {
        let string = string.as_ref();
        let mut buffer = Buffer::new();
        buffer.update(string, size.width() as usize);
        let wrapped_lines = buffer.into_wrapped_lines();
        Self {
            size,
            text_info,
            scroll_position: 0,
            wrapped_lines,
        }
    }
    pub fn up(&mut self) {
        self.scroll_position = self.scroll_position.saturating_sub(1);
    }
    pub fn down(&mut self) {
        let max_scroll = self
            .wrapped_lines
            .len()
            .saturating_sub(self.size.height() as usize + 1);
        self.scroll_position = (self.scroll_position + 1).min(max_scroll);
    }
}

pub struct PagerView;

impl View<Pager> for PagerView {
    fn view<G: ViewGrid>(&mut self, pager: &Pager, offset: Coord, depth: i32, grid: &mut G) {
        let start_index = pager.scroll_position;
        let remaining_lines_to_show = pager
            .wrapped_lines
            .len()
            .saturating_sub(pager.scroll_position);
        let num_lines_to_show = (pager.size.height() as usize).min(remaining_lines_to_show);
        let range = start_index..(start_index + num_lines_to_show);
        for (y_offset, line) in pager.wrapped_lines[range].iter().enumerate() {
            for (x_offset, ch) in line.chars().enumerate() {
                let coord = Coord::new(x_offset as i32, y_offset as i32);
                let view_cell = pager.text_info.view_cell_info(ch);
                grid.set_cell(offset + coord, depth, view_cell);
            }
        }
    }
}

impl ViewSize<Pager> for PagerView {
    fn size(&mut self, pager: &Pager) -> Size {
        pager.size
    }
}
