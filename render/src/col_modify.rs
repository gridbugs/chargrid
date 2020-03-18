use rgb24::Rgb24;

pub trait ColModify: Copy {
    fn foreground(&self, rgb24: Option<Rgb24>) -> Option<Rgb24>;
    fn background(&self, rgb24: Option<Rgb24>) -> Option<Rgb24>;

    fn compose<Other>(self, other: Other) -> ColModifyCompose<Self, Other>
    where
        Other: ColModify,
    {
        ColModifyCompose {
            inner: self,
            outer: other,
        }
    }
}

impl<F: Fn(Option<Rgb24>) -> Option<Rgb24> + Copy> ColModify for F {
    fn foreground(&self, rgb24: Option<Rgb24>) -> Option<Rgb24> {
        (self)(rgb24)
    }
    fn background(&self, rgb24: Option<Rgb24>) -> Option<Rgb24> {
        (self)(rgb24)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ColModifyDefaultForeground(pub Rgb24);
impl ColModify for ColModifyDefaultForeground {
    fn foreground(&self, rgb24: Option<Rgb24>) -> Option<Rgb24> {
        Some(rgb24.unwrap_or(self.0))
    }
    fn background(&self, rgb24: Option<Rgb24>) -> Option<Rgb24> {
        rgb24
    }
}

#[derive(Clone, Copy)]
pub struct ColModifyMap<F: Fn(Rgb24) -> Rgb24 + Copy>(pub F);
impl<F: Fn(Rgb24) -> Rgb24 + Copy> ColModify for ColModifyMap<F> {
    fn foreground(&self, rgb24: Option<Rgb24>) -> Option<Rgb24> {
        rgb24.map(self.0)
    }
    fn background(&self, rgb24: Option<Rgb24>) -> Option<Rgb24> {
        rgb24.map(self.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ColModifyIdentity;

impl ColModify for ColModifyIdentity {
    fn foreground(&self, rgb24: Option<Rgb24>) -> Option<Rgb24> {
        rgb24
    }
    fn background(&self, rgb24: Option<Rgb24>) -> Option<Rgb24> {
        rgb24
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ColModifyCompose<Inner: ColModify, Outer: ColModify> {
    pub inner: Inner,
    pub outer: Outer,
}

impl<Inner, Outer> ColModify for ColModifyCompose<Inner, Outer>
where
    Inner: ColModify,
    Outer: ColModify,
{
    fn foreground(&self, rgb24: Option<Rgb24>) -> Option<Rgb24> {
        self.outer.foreground(self.inner.foreground(rgb24))
    }
    fn background(&self, rgb24: Option<Rgb24>) -> Option<Rgb24> {
        self.outer.background(self.inner.background(rgb24))
    }
}
