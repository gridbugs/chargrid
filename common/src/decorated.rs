/// Represents a particular view `V`, decorated by a decorator `D`.
pub struct Decorated<V, D> {
    pub view: V,
    pub decorator: D,
}
impl<V, D> Decorated<V, D> {
    pub fn new(view: V, decorator: D) -> Self {
        Self { view, decorator }
    }
}
