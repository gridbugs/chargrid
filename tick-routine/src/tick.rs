pub enum Tick<R, C> {
    Return(R),
    Continue(C),
}

impl<R, C> Tick<R, C> {
    pub fn map_continue<D, F>(self, f: F) -> Tick<R, D>
    where
        F: FnOnce(C) -> D,
    {
        match self {
            Tick::Return(r) => Tick::Return(r),
            Tick::Continue(c) => Tick::Continue(f(c)),
        }
    }
    pub fn map_return<S, F>(self, f: F) -> Tick<S, C>
    where
        F: FnOnce(R) -> S,
    {
        match self {
            Tick::Return(r) => Tick::Return(f(r)),
            Tick::Continue(c) => Tick::Continue(c),
        }
    }
}
