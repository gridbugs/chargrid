extern crate prototty_render;
#[cfg(feature = "serialize")]
#[macro_use]
extern crate serde;

mod align;
mod border;
mod bound;
mod defaults;
mod fill_background;
mod identity;
mod scroll;
mod transform_rgb24;

pub use align::*;
pub use border::*;
pub use bound::*;
pub use fill_background::*;
pub use identity::*;
use prototty_render::*;
pub use scroll::*;
pub use transform_rgb24::*;

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
    fn fill_background(self, rgb24: Rgb24) -> FilledBackground<Self> {
        FilledBackground::new(self, rgb24)
    }
    fn transform_rgb24<S>(self, transform_rgb24: S) -> TransformedRgb24<Self, S> {
        TransformedRgb24::new(self, transform_rgb24)
    }
}

impl<V> Decorate for Bounded<V> {}
impl<V> Decorate for Aligned<V> {}
impl<V> Decorate for Bordered<V> {}
impl<V> Decorate for VerticalScrolled<V> {}
impl<V> Decorate for FilledBackground<V> {}
impl<V> Decorate for Identity<V> {}
impl<V, S> Decorate for TransformedRgb24<V, S> {}

pub fn decorate<V>(view: V) -> Identity<V> {
    Identity::new(view)
}
