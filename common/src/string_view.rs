use prototty::*;
use text_info::TextInfo;

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StringView;
impl<T: ?Sized + AsRef<str>> View<T> for StringView {
    fn view<G: ViewGrid>(&mut self, string: &T, offset: Coord, depth: i32, grid: &mut G) {
        let string = string.as_ref();
        for (i, ch) in string.chars().enumerate() {
            if let Some(cell) = grid.get_mut(offset + Coord::new(i as i32, 0), depth) {
                cell.set_character(ch);
            }
        }
    }
}

impl<T: ?Sized + AsRef<str>> ViewSize<T> for StringView {
    fn size(&mut self, string: &T) -> Size {
        let string = string.as_ref();
        Size::new(string.len() as u32, 1)
    }
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RichStringView {
    pub info: TextInfo,
}

impl RichStringView {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn with_info(info: TextInfo) -> Self {
        Self { info }
    }
}

impl<T: ?Sized + AsRef<str>> View<T> for RichStringView {
    fn view<G: ViewGrid>(&mut self, string: &T, offset: Coord, depth: i32, grid: &mut G) {
        let string = string.as_ref();
        for (i, ch) in string.chars().enumerate() {
            if let Some(cell) = grid.get_mut(offset + Coord::new(i as i32, 0), depth) {
                cell.set_character(ch);
                self.info.write_cell(cell);
            }
        }
    }
}

impl<T: ?Sized + AsRef<str>> ViewSize<T> for RichStringView {
    fn size(&mut self, string: &T) -> Size {
        let string = string.as_ref();
        Size::new(string.len() as u32, 1)
    }
}
