use cgmath::Vector2;
use ansi_colour::Colour;
use context::*;
use grid::*;
use defaults::*;

#[derive(Debug, Clone, Copy)]
pub struct TextInfo {
    pub foreground_colour: Colour,
    pub backrgound_colour: Colour,
    pub underline: bool,
    pub bold: bool,
}

impl Default for TextInfo {
    fn default() -> Self {
        Self {
            foreground_colour: DEFAULT_FG,
            backrgound_colour: DEFAULT_BG,
            underline: false,
            bold: false,
        }
    }
}

impl TextInfo {
    pub fn foreground_colour(self, colour: Colour) -> Self {
        Self { foreground_colour: colour, .. self }
    }
    pub fn backrgound_colour(self, colour: Colour) -> Self {
        Self { backrgound_colour: colour, .. self }
    }
    pub fn underline(self) -> Self {
        Self { underline: true, .. self }
    }
    pub fn bold(self) -> Self {
        Self { bold: true, .. self }
    }
}

#[derive(Debug, Clone)]
struct RichTextPart {
    text: String,
    info: TextInfo,
}

#[derive(Debug, Clone)]
struct RichTextInner {
    size: Vector2<u16>,
    parts: Vec<RichTextPart>,
}

impl RichTextInner {
    fn new<D: Into<Vector2<u16>>>(parts: &[(&str, TextInfo)], size: D) -> Self {
        Self {
            size: size.into(),
            parts: parts.iter().map(|&(ref s, ref i)| {
                RichTextPart {
                    text: s.to_string(),
                    info: i.clone(),
                }
            }).collect(),
        }
    }
    fn render(&self, grid: &mut Grid<Cell>, seq: u64, offset: Vector2<i16>, depth: i16) {
        let bottom_right_abs = offset + self.size.cast();
        let mut coord = offset;
        'part_loop: for part in self.parts.iter() {
            for ch in part.text.chars() {
                match ch {
                    '\n' => {
                        coord.x = offset.x;
                        coord.y += 1;
                        if coord.y == bottom_right_abs.y {
                            break;
                        }
                    }
                    '\r' => {
                        coord.x = offset.x;
                    }
                    _ => {
                        if let Some(cell) = grid.get_mut(coord) {
                            cell.update_with_style(seq, ch, depth,
                                                   part.info.foreground_colour,
                                                   part.info.backrgound_colour,
                                                   part.info.bold,
                                                   part.info.underline);
                        }
                        coord.x += 1;
                        if coord.x == bottom_right_abs.x {
                            coord.x = offset.x;
                            coord.y += 1;
                            if coord.y == bottom_right_abs.y {
                                break 'part_loop;
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct RichText(ElementCell<RichTextInner>);

impl RichText {
    pub fn new<D: Into<Vector2<u16>>>(parts: &[(&str, TextInfo)], size: D) -> Self {
        RichText(element_cell(RichTextInner::new(parts, size)))
    }
    pub fn size(&self) -> Vector2<u16> {
        (*self.0).borrow().size
    }
    pub(crate) fn render(&self, grid: &mut Grid<Cell>, seq: u64, offset: Vector2<i16>, depth: i16) {
        (*self.0).borrow().render(grid, seq, offset, depth);
    }
}
