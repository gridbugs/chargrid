use prototty_render::Style;
use std::marker::PhantomData;

pub struct MenuEntryToRender<'a, E> {
    pub entry: &'a E,
    pub selected: bool,
    pub index: usize,
}

pub trait MenuEntryString {
    type Entry;
    fn render_string(&self, entry: MenuEntryToRender<Self::Entry>, buf: &mut String);
}

#[derive(Clone, Copy)]
pub struct MenuEntryStringIntoStr<E>(PhantomData<E>)
where
    for<'a> &'a E: Into<&'a str>;
impl<E> MenuEntryStringIntoStr<E>
where
    for<'a> &'a E: Into<&'a str>,
{
    pub fn new() -> Self {
        Self(PhantomData)
    }
}
impl<E> MenuEntryString for MenuEntryStringIntoStr<E>
where
    for<'a> &'a E: Into<&'a str>,
{
    type Entry = E;
    fn render_string(&self, entry: MenuEntryToRender<Self::Entry>, buf: &mut String) {
        use std::fmt::Write;
        write!(buf, "{}", entry.entry.into()).unwrap()
    }
}

pub struct MenuEntryStringFn<F, E> {
    f: F,
    e: PhantomData<E>,
}
impl<F, E> MenuEntryStringFn<F, E> {
    pub fn new(f: F) -> Self {
        Self { f, e: PhantomData }
    }
}
impl<F, E> MenuEntryString for MenuEntryStringFn<F, E>
where
    F: Fn(MenuEntryToRender<E>, &mut String),
{
    type Entry = E;
    fn render_string(&self, entry: MenuEntryToRender<Self::Entry>, buf: &mut String) {
        (self.f)(entry, buf);
    }
}

pub trait MenuEntryRichString {
    type Entry;
    fn render_rich_string(&self, entry: MenuEntryToRender<Self::Entry>, buf: &mut String) -> Style;
}

pub struct MenuEntryRichStringFn<F, E> {
    f: F,
    e: PhantomData<E>,
}
impl<F, E> MenuEntryRichStringFn<F, E> {
    pub fn new(f: F) -> Self {
        Self { f, e: PhantomData }
    }
}
impl<F, E> MenuEntryRichString for MenuEntryRichStringFn<F, E>
where
    F: Fn(MenuEntryToRender<E>, &mut String) -> Style,
{
    type Entry = E;
    fn render_rich_string(&self, entry: MenuEntryToRender<Self::Entry>, buf: &mut String) -> Style {
        (self.f)(entry, buf)
    }
}
