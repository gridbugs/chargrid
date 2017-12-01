use prototty::*;
use cgmath::Vector2;
use ansi_colour::Colour;
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
pub struct RichTextPart {
    pub string: String,
    pub info: TextInfo,
}

impl<S: Into<String>> From<(S, TextInfo)> for RichTextPart {
    fn from((string, info): (S, TextInfo)) -> Self {
        RichTextPart {
            string: string.into(),
            info,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RichText {
    pub parts: Vec<RichTextPart>,
    pub size: Vector2<u16>,
}

impl RichText {
    pub fn new<S, V>(mut parts: Vec<(S, TextInfo)>, size: V) -> Self
        where S: Into<String>,
              V: Into<Vector2<u16>>,
    {
        Self {
            parts: parts.drain(..).map(Into::into).collect(),
            size: size.into(),
        }
    }
}

impl View for RichText {
    fn view<G: ViewGrid>(&self, offset: Vector2<i16>, depth: i16, grid: &mut G) {
        let bottom_right_abs = offset + self.size.cast();
        let mut coord = offset;
        'part_loop: for part in self.parts.iter() {
            for ch in part.string.chars() {
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
                            cell.update_with_style(ch, depth,
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
    fn size(&self) -> Vector2<u16> { self.size }
}
