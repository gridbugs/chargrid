use prototty::*;
use text_info::*;

/// A section of text sharing a common `TextInfo`.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// A text element, where the style of the text
/// can be controlled. A single `RichText` element can have
/// several different parts, each styled differently.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichText {
    pub parts: Vec<RichTextPart>,
    pub size: Size,
}

impl RichText {
    /// Create a new `RichText` element.
    pub fn new<S>(mut parts: Vec<(S, TextInfo)>, size: Size) -> Self
        where S: Into<String>,
    {
        Self {
            parts: parts.drain(..).map(Into::into).collect(),
            size,
        }
    }

    /// Create a new `Text` element of an appropriate
    /// size for a single line.
    pub fn one_line<S>(mut parts: Vec<(S, TextInfo)>) -> Self
        where S: Into<String>,
    {
        let parts: Vec<RichTextPart> = parts.drain(..).map(Into::into).collect();
        let length = parts.iter().fold(0, |acc, part| acc + part.string.len());
        Self {
            parts,
            size: Size::new(length as u32, 1),
        }
    }
}

/// Default view of a `RichText`.
pub struct DefaultRichTextView;

impl View<RichText> for DefaultRichTextView {
    fn view<G: ViewGrid>(&self, data: &RichText, offset: Coord, depth: i32, grid: &mut G) {
        let bottom_right_abs = offset + data.size;
        let mut coord = offset;
        'part_loop: for part in data.parts.iter() {
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
                        if let Some(cell) = grid.get_mut(coord, depth) {
                            cell.character = ch;
                            part.info.write_cell(cell);
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

impl ViewSize<RichText> for DefaultRichTextView {
    fn size(&self, data: &RichText) -> Size { data.size }
}
