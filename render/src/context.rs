use super::{Coord, Size};
use rgb24::Rgb24;

pub trait ViewTransformRgb24: Copy {
    fn transform(&self, rgb24: Rgb24) -> Rgb24;
}

impl<F: Fn(Rgb24) -> Rgb24 + Copy> ViewTransformRgb24 for F {
    fn transform(&self, rgb24: Rgb24) -> Rgb24 {
        (self)(rgb24)
    }
}

#[derive(Clone, Copy)]
pub struct ViewTransformRgb24Identity;

impl ViewTransformRgb24 for ViewTransformRgb24Identity {
    fn transform(&self, rgb24: Rgb24) -> Rgb24 {
        rgb24
    }
}

#[derive(Clone, Copy)]
pub struct ViewTransformRgb24Compose<Inner: ViewTransformRgb24, Outer: ViewTransformRgb24>
{
    inner: Inner,
    outer: Outer,
}

impl<Inner, Outer> ViewTransformRgb24 for ViewTransformRgb24Compose<Inner, Outer>
where
    Inner: ViewTransformRgb24,
    Outer: ViewTransformRgb24,
{
    fn transform(&self, rgb24: Rgb24) -> Rgb24 {
        self.outer.transform(self.inner.transform(rgb24))
    }
}

#[derive(Clone, Copy)]
pub struct ViewContext<R: ViewTransformRgb24 = ViewTransformRgb24Identity> {
    pub inner_offset: Coord,
    pub outer_offset: Coord,
    pub depth: i32,
    pub transform_rgb24: R,
    pub size: Size,
}

pub type ViewContextDefault = ViewContext<ViewTransformRgb24Identity>;

impl ViewContext<ViewTransformRgb24Identity> {
    pub fn default_with_size(size: Size) -> Self {
        Self {
            inner_offset: Coord::new(0, 0),
            outer_offset: Coord::new(0, 0),
            depth: 0,
            transform_rgb24: ViewTransformRgb24Identity,
            size,
        }
    }
}

impl<R: ViewTransformRgb24> ViewContext<R> {
    pub fn new(
        inner_offset: Coord,
        outer_offset: Coord,
        depth: i32,
        transform_rgb24: R,
        size: Size,
    ) -> Self {
        Self {
            inner_offset,
            outer_offset,
            depth,
            transform_rgb24,
            size,
        }
    }

    pub fn add_inner_offset(self, offset_delta: Coord) -> Self {
        Self {
            inner_offset: self.inner_offset + offset_delta,
            ..self
        }
    }

    pub fn add_offset(self, offset_delta: Coord) -> Self {
        Self {
            outer_offset: self.outer_offset + offset_delta,
            size: (self.size.to_coord().unwrap() - offset_delta)
                .to_size()
                .unwrap_or(Size::new_u16(0, 0)),
            ..self
        }
    }

    pub fn add_depth(self, depth_delta: i32) -> Self {
        Self {
            depth: self.depth + depth_delta,
            ..self
        }
    }

    pub fn constrain_size_to(self, size: Size) -> Self {
        Self {
            size: Size::new(self.size.x().min(size.x()), self.size.y().min(size.y())),
            ..self
        }
    }

    pub fn constrain_size_by(self, size: Size) -> Self {
        Self {
            size: self.size.saturating_sub(size),
            ..self
        }
    }

    pub fn compose_transform_rgb24<Inner: ViewTransformRgb24>(
        self,
        inner: Inner,
    ) -> ViewContext<ViewTransformRgb24Compose<Inner, R>> {
        ViewContext {
            transform_rgb24: ViewTransformRgb24Compose {
                inner,
                outer: self.transform_rgb24,
            },
            inner_offset: self.inner_offset,
            outer_offset: self.outer_offset,
            depth: self.depth,
            size: self.size,
        }
    }
}
