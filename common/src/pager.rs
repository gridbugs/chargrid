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
            if self.wrapped_lines[index_of_current_line].len() + self.current_word.len()
                <= width
            {
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
    /// panics if `size.width() == 0`
    pub fn new<T: AsRef<str>>(string: T, size: Size, text_info: TextInfo) -> Self {
        if size.width() == 0 {
            panic!("may not have 0 width");
        }
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
        let height = self.size.height() as usize;
        self.scroll_up_lines(height);
    }
    pub fn scroll_down_page(&mut self) {
        let height = self.size.height() as usize;
        self.scroll_down_lines(height);
    }
    pub fn scroll_to_top(&mut self) {
        self.scroll_position = 0;
    }
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_position = self.max_scroll_position();
    }
    pub fn max_scroll_position(&self) -> usize {
        self.wrapped_lines
            .len()
            .saturating_sub(self.size.height() as usize + 1)
    }
    pub fn scroll_position(&self) -> usize {
        self.scroll_position
    }
    pub fn size(&self) -> Size {
        self.size
    }
    pub fn num_lines(&self) -> usize {
        self.wrapped_lines.len()
    }
}

pub struct PagerScrollbar {
    pub text_info: TextInfo,
    pub character: char,
    pub padding: u32,
}

impl Default for PagerScrollbar {
    fn default() -> Self {
        Self {
            text_info: TextInfo::default(),
            character: '█',
            padding: 1,
        }
    }
}

impl PagerScrollbar {
    pub fn new(text_info: TextInfo, character: char, padding: u32) -> Self {
        Self {
            text_info,
            character,
            padding,
        }
    }
    pub fn padding(&self) -> u32 {
        self.padding
    }
}

pub struct PagerView;

impl View<Pager> for PagerView {
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        pager: &Pager,
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        let start_index = pager.scroll_position;
        let remaining_lines_to_show = pager
            .wrapped_lines
            .len()
            .saturating_sub(pager.scroll_position);
        let num_lines_to_show =
            (pager.size.height() as usize).min(remaining_lines_to_show);
        let range = start_index..(start_index + num_lines_to_show);
        for (y_offset, line) in pager.wrapped_lines[range].iter().enumerate() {
            for (x_offset, ch) in line.chars().enumerate() {
                let coord = Coord::new(x_offset as i32, y_offset as i32);
                let view_cell = pager.text_info.view_cell_info(ch);
                grid.set_cell_relative(coord, 0, view_cell, context);
            }
        }
    }
}

impl ViewSize<Pager> for PagerView {
    fn size(&mut self, pager: &Pager) -> Size {
        pager.size
    }
}

pub struct PagerViewWithScrollbar<V>(pub V);

impl<'a, V: View<Pager>> View<(&'a Pager, &'a PagerScrollbar)>
    for PagerViewWithScrollbar<V>
{
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        data: &(&'a Pager, &'a PagerScrollbar),
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        self.0.view(data.0, context, grid);
        let pager_height = data.0.size().height();
        if data.0.num_lines() as u32 > pager_height {
            let view_cell = data.1.text_info.view_cell_info(data.1.character);
            let bar_x = data.0.size().width() as i32 + data.1.padding as i32;
            let bar_height = (pager_height * pager_height) / data.0.num_lines() as u32;
            let bar_top = ((pager_height - bar_height) * data.0.scroll_position() as u32)
                / data.0.max_scroll_position() as u32;
            for y in 0..bar_height {
                let bar_y = (y + bar_top) as i32;
                let coord = Coord::new(bar_x, bar_y);
                grid.set_cell_relative(coord, 0, view_cell, context);
            }
        }
    }
}

impl<'a, V: ViewSize<Pager>> ViewSize<(&'a Pager, &'a PagerScrollbar)>
    for PagerViewWithScrollbar<V>
{
    fn size(&mut self, data: &(&'a Pager, &'a PagerScrollbar)) -> Size {
        self.0
            .size(data.0)
            .saturating_sub(Size::new(data.1.padding + 1, 0))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn wrap_lines(string: &str, width: u32) -> Vec<String> {
        let pager = Pager::new(string, Size::new(width, 100), Default::default());
        pager.wrapped_lines
    }

    #[test]
    fn small_example() {
        let lines = wrap_lines("Hello, World!", 8);
        assert_eq!(lines, ["Hello, ", "World!"]);
    }

    #[test]
    fn single_long_word_8() {
        let lines = wrap_lines("abcdefghijklmnopqrstuvwxyz", 8);
        assert_eq!(lines, ["abcdefgh", "ijklmnop", "qrstuvwx", "yz"]);
    }

    #[test]
    fn single_long_word_1() {
        let lines = wrap_lines("abcdefghijklmnop", 1);
        assert_eq!(
            lines,
            [
                "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n",
                "o", "p",
            ]
        );
    }

    #[test]
    fn consecutive_spaces() {
        let lines =
            wrap_lines("hello        world     there  are lots  of    spaces", 10);
        assert_eq!(
            lines,
            [
                "hello     ",
                "world     ",
                "there  are",
                "lots  of  ",
                "spaces"
            ]
        );
    }

    #[test]
    fn newlines() {
        let lines = wrap_lines("hello\nworld", 100);
        assert_eq!(lines, ["hello", "world"]);
    }

    #[test]
    fn empty() {
        let lines = wrap_lines("", 100);
        assert_eq!(lines, [""]);
    }

    #[test]
    fn only_spaces() {
        let lines = wrap_lines("      ", 100);
        assert_eq!(lines, [""]);
    }

}
