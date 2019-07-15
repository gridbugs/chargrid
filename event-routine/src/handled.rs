pub enum Handled<R, C> {
    Return(R),
    Continue(C),
}

impl<R, C> Handled<R, C> {
    pub fn map_continue<D, F>(self, f: F) -> Handled<R, D>
    where
        F: FnOnce(C) -> D,
    {
        match self {
            Handled::Return(r) => Handled::Return(r),
            Handled::Continue(c) => Handled::Continue(f(c)),
        }
    }
    pub fn map_return<S, F>(self, f: F) -> Handled<S, C>
    where
        F: FnOnce(R) -> S,
    {
        match self {
            Handled::Return(r) => Handled::Return(f(r)),
            Handled::Continue(c) => Handled::Continue(c),
        }
    }
}
