use prototty_render::*;

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub enum AlignX {
    Left,
    Centre,
    Right,
}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub enum AlignY {
    Top,
    Centre,
    Bottom,
}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct Align {
    pub size: Size,
    pub x: AlignX,
    pub y: AlignY,
}

impl Align {
    pub fn new(size: Size, x: AlignX, y: AlignY) -> Self {
        Self { size, x, y }
    }
}

pub struct Aligned<V> {
    pub view: V,
    pub align: Align,
}

impl<V> Aligned<V> {
    pub fn new(view: V, align: Align) -> Self {
        Self { view, align }
    }
}

impl<T: ?Sized, V: View<T> + ViewSize<T>> View<T> for Aligned<V> {
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        data: &T,
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        let data_size = self.view.size(data);
        let x_offset = match self.align.x {
            AlignX::Left => 0,
            AlignX::Centre => (self.align.size.x() as i32 - data_size.x() as i32) / 2,
            AlignX::Right => self.align.size.x() as i32 - data_size.x() as i32,
        };
        let y_offset = match self.align.y {
            AlignY::Top => 0,
            AlignY::Centre => (self.align.size.y() as i32 - data_size.y() as i32) / 2,
            AlignY::Bottom => self.align.size.y() as i32 - data_size.y() as i32,
        };
        self.view.view(
            data,
            context
                .add_offset(Coord::new(x_offset, y_offset))
                .constrain_size(data_size),
            grid,
        );
    }
}

impl<T: ?Sized, V: ViewSize<T>> ViewSize<T> for Aligned<V> {
    fn size(&mut self, _data: &T) -> Size {
        self.align.size
    }
}
