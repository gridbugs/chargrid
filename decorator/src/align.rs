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
pub struct AlignView<V> {
    pub view: V,
}

impl<V> AlignView<V> {
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

impl<'a, T, V: View<&'a T>> View<&'a AlignData<T>> for AlignView<V> {
    fn view<F: Frame, C: ColModify>(
        &mut self,
        &AlignData { alignment, ref data }: &'a AlignData<T>,
        context: ViewContext<C>,
        frame: &mut F,
    ) {
        self.view(AlignData { alignment, data }, context, frame);
    }
}

impl<T: Clone, V: View<T>> View<AlignData<T>> for AlignView<V> {
    fn view<F: Frame, C: ColModify>(
        &mut self,
        AlignData { alignment, data }: AlignData<T>,
        context: ViewContext<C>,
        frame: &mut F,
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
        self.view
            .view(data, context.add_offset(Coord::new(x_offset, y_offset)), frame);
    }
}

pub struct AlignView_<'v, V> {
    pub view: &'v mut V,
    pub alignment: Alignment,
}

impl<'v, V, T> View<T> for AlignView_<'v, V>
where
    V: View<T>,
    T: Clone,
{
    fn view<F: Frame, C: ColModify>(&mut self, data: T, context: ViewContext<C>, frame: &mut F) {
        let data_size = self.view.visible_bounds(data.clone(), context);
        let x_offset = match self.alignment.x {
            AlignmentX::Left => 0,
            AlignmentX::Centre => (context.size.x() as i32 - data_size.x() as i32) / 2,
            AlignmentX::Right => context.size.x() as i32 - data_size.x() as i32,
        };
        let y_offset = match self.alignment.y {
            AlignmentY::Top => 0,
            AlignmentY::Centre => (context.size.y() as i32 - data_size.y() as i32) / 2,
            AlignmentY::Bottom => context.size.y() as i32 - data_size.y() as i32,
        };
        self.view
            .view(data, context.add_offset(Coord::new(x_offset, y_offset)), frame);
    }
}
