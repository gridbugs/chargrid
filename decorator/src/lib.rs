extern crate prototty_render;
#[cfg(feature = "serialize")]
#[macro_use]
extern crate serde;

mod align;
mod border;
mod bound;
mod defaults;
mod identity;
mod scroll;

pub use align::*;
pub use border::*;
pub use bound::*;
pub use identity::*;
use prototty_render::*;
pub use scroll::*;

pub trait Decorate: Sized {
    fn bound(self, size: Size) -> Bounded<Self> {
        Bounded::new(self, size)
    }
    fn align(self, align: Align) -> Aligned<Self> {
        Aligned::new(self, align)
    }
    fn centre(self) -> Aligned<Self> {
        Aligned::centre(self)
    }
    fn border(self, border: Border) -> Bordered<Self> {
        Bordered::new(self, border)
    }
    fn vertical_scroll(self, scrollbar: VerticalScrollbar) -> VerticalScrolled<Self> {
        VerticalScrolled::new(self, scrollbar)
    }
}

impl<V> Decorate for Bounded<V> {}
impl<V> Decorate for Aligned<V> {}
impl<V> Decorate for Bordered<V> {}
impl<V> Decorate for VerticalScrolled<V> {}
impl<V> Decorate for Identity<V> {}

pub fn decorate<V>(view: V) -> Identity<V> {
    Identity::new(view)
}
