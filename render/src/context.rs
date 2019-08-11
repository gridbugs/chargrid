use super::{Coord, Size};
use rgb24::Rgb24;

pub trait ColModify: Copy {
    fn modify(&self, rgb24: Rgb24) -> Rgb24;
}

impl<F: Fn(Rgb24) -> Rgb24 + Copy> ColModify for F {
    fn modify(&self, rgb24: Rgb24) -> Rgb24 {
        (self)(rgb24)
    }
}

#[derive(Clone, Copy)]
pub struct ColModifyIdentity;

impl ColModify for ColModifyIdentity {
    fn modify(&self, rgb24: Rgb24) -> Rgb24 {
        rgb24
    }
}

#[derive(Clone, Copy)]
pub struct ColModifyCompose<Inner: ColModify, Outer: ColModify> {
    inner: Inner,
    outer: Outer,
}

impl<Inner, Outer> ColModify for ColModifyCompose<Inner, Outer>
where
    Inner: ColModify,
    Outer: ColModify,
{
    fn modify(&self, rgb24: Rgb24) -> Rgb24 {
        self.outer.modify(self.inner.modify(rgb24))
    }
}

#[derive(Clone, Copy)]
pub struct ViewContext<C: ColModify = ColModifyIdentity> {
    pub offset: Coord,
    pub depth: i32,
    pub col_modify: C,
    pub size: Size,
}

pub type ViewContextDefault = ViewContext<ColModifyIdentity>;

impl ViewContext<ColModifyIdentity> {
    pub fn default_with_size(size: Size) -> Self {
        Self {
            offset: Coord::new(0, 0),
            depth: 0,
            col_modify: ColModifyIdentity,
            size,
        }
    }
}

impl<C: ColModify> ViewContext<C> {
    pub fn new(offset: Coord, depth: i32, col_modify: C, size: Size) -> Self {
        Self {
            offset,
            depth,
            col_modify,
            size,
        }
    }

    pub fn add_offset(self, offset_delta: Coord) -> Self {
        Self {
            offset: self.offset + offset_delta,
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

    pub fn compose_col_modify<Inner: ColModify>(self, inner: Inner) -> ViewContext<ColModifyCompose<Inner, C>> {
        ViewContext {
            col_modify: ColModifyCompose {
                inner,
                outer: self.col_modify,
            },
            offset: self.offset,
            depth: self.depth,
            size: self.size,
        }
    }
}
