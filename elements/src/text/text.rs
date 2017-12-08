use prototty::*;
use cgmath::Vector2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Text {
    pub string: String,
    pub size: Vector2<u16>,
}

impl Text {
    pub fn new<S, V>(string: S, size: V) -> Self
        where S: Into<String>,
              V: Into<Vector2<u16>>,
    {
        Self {
            string: string.into(),
            size: size.into(),
        }
    }
    pub fn one_line<S>(string: S) -> Self
        where S: Into<String>,
    {
        let string = string.into();
        Self {
            size: Vector2::new(string.len() as u16, 1),
            string,
        }
    }
}

impl View for Text {
    fn view<G: ViewGrid>(&self, offset: Vector2<i16>, depth: i16, grid: &mut G) {
        let bottom_right_abs = offset + self.size.cast();
        let mut coord = offset;
        for ch in self.string.chars() {
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
                        cell.update(ch, depth);
                    }
                    coord.x += 1;
                    if coord.x == bottom_right_abs.x {
                        coord.x = offset.x;
                        coord.y += 1;
                        if coord.y == bottom_right_abs.y {
                            break;
                        }
                    }
                }
            }
        }
    }
}

impl ViewSize for Text {
    fn size(&self) -> Vector2<u16> { self.size }
}
