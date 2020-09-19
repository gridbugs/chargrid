use super::{Coord, Size};
use crate::col_modify::{ColModify, ColModifyCompose, ColModifyIdentity};

#[derive(Clone, Copy, Debug)]
pub struct ViewContext<C: ColModify = ColModifyIdentity> {
    pub offset: Coord,
    pub depth: i8,
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
    pub fn new(offset: Coord, depth: i8, col_modify: C, size: Size) -> Self {
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
                .unwrap_or_else(|coord_2d::NegativeDimension| Size::new_u16(0, 0)),
            ..self
        }
    }

    pub fn add_depth(self, depth_delta: i8) -> Self {
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

    pub fn compose_col_modify<Inner: ColModify>(
        self,
        inner: Inner,
    ) -> ViewContext<ColModifyCompose<Inner, C>> {
        ViewContext {
            col_modify: inner.compose(self.col_modify),
            offset: self.offset,
            depth: self.depth,
            size: self.size,
        }
    }
}
