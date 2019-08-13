use crate::instance::*;
use prototty_render::*;
use prototty_text::StringViewSingleLine;

pub struct MenuInstanceView<E> {
    pub entry_view: E,
    last_offset: Coord,
    last_size: Size,
}

impl<E> MenuInstanceView<E> {
    pub const fn new(entry_view: E) -> Self {
        Self {
            entry_view,
            last_offset: Coord::new(0, 0),
            last_size: Size::new_u16(0, 0),
        }
    }
}

impl<E> MenuIndexFromScreenCoord for MenuInstanceView<E> {
    fn menu_index_from_screen_coord(&self, len: usize, coord: Coord) -> Option<usize> {
        let rel_coord = coord - self.last_offset;
        if rel_coord.x < 0 || rel_coord.y < 0 || rel_coord.x >= self.last_size.x() as i32 || rel_coord.y >= len as i32 {
            None
        } else {
            Some(rel_coord.y as usize)
        }
    }
}

impl<'a, T, E> View<&'a MenuInstance<T>> for MenuInstanceView<E>
where
    T: Clone,
    E: MenuEntryView<T>,
{
    fn view<F: Frame, C: ColModify>(
        &mut self,
        menu_instance: &'a MenuInstance<T>,
        context: ViewContext<C>,
        frame: &mut F,
    ) {
        self.last_offset = context.offset;
        let mut max_width = 0;
        for (i, entry) in menu_instance.menu.iter().enumerate() {
            let width = if i == menu_instance.selected_index {
                self.entry_view
                    .selected(entry, context.add_offset(Coord::new(0, i as i32)), frame)
            } else {
                self.entry_view
                    .normal(entry, context.add_offset(Coord::new(0, i as i32)), frame)
            };
            max_width = max_width.max(width);
        }
        self.last_size = Size::new(max_width, menu_instance.menu.len() as u32);
    }
}

impl<'a, T, E> View<(&'a MenuInstance<T>, &'a E::Extra)> for MenuInstanceView<E>
where
    T: Clone,
    E: MenuEntryExtraView<T>,
{
    fn view<F: Frame, C: ColModify>(
        &mut self,
        (menu_instance, extra): (&'a MenuInstance<T>, &'a E::Extra),
        context: ViewContext<C>,
        frame: &mut F,
    ) {
        self.last_offset = context.offset;
        let mut max_width = 0;
        for (i, entry) in menu_instance.menu.iter().enumerate() {
            let width = if i == menu_instance.selected_index {
                self.entry_view
                    .selected(entry, extra, context.add_offset(Coord::new(0, i as i32)), frame)
            } else {
                self.entry_view
                    .normal(entry, extra, context.add_offset(Coord::new(0, i as i32)), frame)
            };
            max_width = max_width.max(width);
        }
        self.last_size = Size::new(max_width, menu_instance.menu.len() as u32);
    }
}

pub type MenuEntryViewInfo = u32;

pub trait MenuEntryView<T> {
    fn normal<F: Frame, C: ColModify>(
        &mut self,
        entry: &T,
        context: ViewContext<C>,
        frame: &mut F,
    ) -> MenuEntryViewInfo;
    fn selected<F: Frame, C: ColModify>(
        &mut self,
        entry: &T,
        context: ViewContext<C>,
        frame: &mut F,
    ) -> MenuEntryViewInfo;
}

/// Sometimes the menu entry alone is not sufficient to render the
/// menu entry. An example is when the mappings from menu entry
/// to (say) string is not statically known. The `extra` argument
/// to the methods of this trait can be used to pass some external
/// object which knows how to map menu entries to some renderable
/// value.
pub trait MenuEntryExtraView<T> {
    type Extra;
    fn normal<F: Frame, C: ColModify>(
        &mut self,
        entry: &T,
        extra: &Self::Extra,
        context: ViewContext<C>,
        frame: &mut F,
    ) -> MenuEntryViewInfo;
    fn selected<F: Frame, C: ColModify>(
        &mut self,
        entry: &T,
        extra: &Self::Extra,
        context: ViewContext<C>,
        frame: &mut F,
    ) -> MenuEntryViewInfo;
}

/// Convenience function to simplify implementing `MenuEntryView` and
/// `MenuEntryExtraView`.
pub fn menu_entry_view<T, V: View<T>, F: Frame, C: ColModify>(
    data: T,
    mut view: V,
    context: ViewContext<C>,
    frame: &mut F,
) -> MenuEntryViewInfo {
    view.view_size(data, context, frame).width()
}

/// An implementation of `MenuEntryView` for menus whose entries
/// can be converted into string slices.
pub struct MenuEntryStylePair {
    pub normal: Style,
    pub selected: Style,
}

impl MenuEntryStylePair {
    pub const fn new(normal: Style, selected: Style) -> Self {
        Self { normal, selected }
    }
    pub const fn default() -> Self {
        Self::new(Style::new(), Style::new().with_bold(true))
    }
}

impl<T> MenuEntryView<T> for MenuEntryStylePair
where
    for<'a> &'a T: Into<&'a str>,
{
    fn normal<F: Frame, C: ColModify>(
        &mut self,
        entry: &T,
        context: ViewContext<C>,
        frame: &mut F,
    ) -> MenuEntryViewInfo {
        menu_entry_view(entry.into(), StringViewSingleLine::new(self.normal), context, frame)
    }
    fn selected<F: Frame, C: ColModify>(
        &mut self,
        entry: &T,
        context: ViewContext<C>,
        frame: &mut F,
    ) -> MenuEntryViewInfo {
        menu_entry_view(entry.into(), StringViewSingleLine::new(self.selected), context, frame)
    }
}

pub trait ChooseStyleFromEntryExtra {
    type Extra;
    type Entry;
    fn choose_style_normal(&mut self, entry: &Self::Entry, extra: &Self::Extra) -> Style;
    fn choose_style_selected(&mut self, entry: &Self::Entry, extra: &Self::Extra) -> Style;
}

impl<T, CS> MenuEntryExtraView<T> for CS
where
    for<'a> &'a T: Into<&'a str>,
    CS: ChooseStyleFromEntryExtra<Entry = T>,
{
    type Extra = CS::Extra;
    fn normal<G: Frame, C: ColModify>(
        &mut self,
        entry: &T,
        extra: &Self::Extra,
        context: ViewContext<C>,
        frame: &mut G,
    ) -> MenuEntryViewInfo {
        menu_entry_view(
            entry.into(),
            StringViewSingleLine::new(self.choose_style_normal(entry, extra)),
            context,
            frame,
        )
    }
    fn selected<G: Frame, C: ColModify>(
        &mut self,
        entry: &T,
        extra: &Self::Extra,
        context: ViewContext<C>,
        frame: &mut G,
    ) -> MenuEntryViewInfo {
        menu_entry_view(
            entry.into(),
            StringViewSingleLine::new(self.choose_style_selected(entry, extra)),
            context,
            frame,
        )
    }
}
