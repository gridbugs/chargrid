use prototty_render::*;

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub enum AlignmentX {
    Left,
    Centre,
    Right,
}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub enum AlignmentY {
    Top,
    Centre,
    Bottom,
}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct Alignment {
    pub x: AlignmentX,
    pub y: AlignmentY,
}

impl Alignment {
    pub fn new(x: AlignmentX, y: AlignmentY) -> Self {
        Self { x, y }
    }
    pub fn centre() -> Self {
        Self::new(AlignmentX::Centre, AlignmentY::Centre)
    }
}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct Align<V> {
    pub view: V,
}

impl<V> Align<V> {
    pub fn new(view: V) -> Self {
        Self { view }
    }
}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct AlignData<T> {
    pub alignment: Alignment,
    pub data: T,
}

impl<T> AlignData<T> {
    pub fn new(alignment: Alignment, data: T) -> Self {
        Self { alignment, data }
    }
}

impl<T: Clone, V: View<T>> View<(Alignment, T)> for Align<V> {
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        (alignment, data): (Alignment, T),
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        self.view(AlignData::new(alignment, data), context, grid);
    }
}

impl<T: Clone, V: View<T>> View<AlignData<T>> for Align<V> {
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        AlignData { data, alignment }: AlignData<T>,
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        let data_size = self.view.visible_bounds(data.clone(), context);
        let x_offset = match alignment.x {
            AlignmentX::Left => 0,
            AlignmentX::Centre => (context.size.x() as i32 - data_size.x() as i32) / 2,
            AlignmentX::Right => context.size.x() as i32 - data_size.x() as i32,
        };
        let y_offset = match alignment.y {
            AlignmentY::Top => 0,
            AlignmentY::Centre => (context.size.y() as i32 - data_size.y() as i32) / 2,
            AlignmentY::Bottom => context.size.y() as i32 - data_size.y() as i32,
        };
        self.view.view(
            data,
            context.add_offset(Coord::new(x_offset, y_offset)),
            grid,
        );
    }
}
