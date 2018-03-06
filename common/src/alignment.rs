use prototty::*;
use decorated::Decorated;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Alignment {
    TopLeft,
    Centre,
    BottomRight,
}

pub struct Align {
    pub size: Size,
    pub x_alignment: Alignment,
    pub y_alignment: Alignment,
}

impl Align {
    pub fn new(size: Size, x_alignment: Alignment, y_alignment: Alignment) -> Self {
        Self {
            size,
            x_alignment,
            y_alignment,
        }
    }
}

impl<T, V: View<T> + ViewSize<T>> ViewSize<T> for Decorated<V, Align> {
    fn size(&mut self, data: &T) -> Size {
        let data_size = self.view.size(data);
        let x = ::std::cmp::max(data_size.x(), self.decorator.size.x());
        let y = ::std::cmp::max(data_size.y(), self.decorator.size.y());
        Size::new(x, y)
    }
}

impl<T, V: View<T> + ViewSize<T>> View<T> for Decorated<V, Align> {
    fn view<G: ViewGrid>(&mut self, data: &T, offset: Coord, depth: i32, grid: &mut G) {
        let data_size = self.view.size(data);

        let x_offset = if self.decorator.size.x() > data_size.x() {
            match self.decorator.x_alignment {
                Alignment::TopLeft => 0,
                Alignment::Centre => (self.decorator.size.x() - data_size.x()) / 2,
                Alignment::BottomRight => self.decorator.size.x() - data_size.x(),
            }
        } else {
            0
        };

        let y_offset = if self.decorator.size.y() > data_size.y() {
            match self.decorator.y_alignment {
                Alignment::TopLeft => 0,
                Alignment::Centre => (self.decorator.size.y() - data_size.y()) / 2,
                Alignment::BottomRight => self.decorator.size.y() - data_size.y(),
            }
        } else {
            0
        };

        self.view.view(
            data,
            offset + Coord::new(x_offset as i32, y_offset as i32),
            depth,
            grid,
        );
    }
}
