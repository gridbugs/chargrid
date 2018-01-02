/// Represents a particular view `V`, decorated by a decorator `D`.
pub struct Decorated<'a, 'b, V: 'a, D: 'b> {
    pub view: &'a V,
    pub decorator: &'b D,
}
impl<'a, 'b, V, D> Decorated<'a, 'b, V, D> {
    pub fn new(view: &'a V, decorator: &'b D) -> Self {
        Self { view, decorator }
    }
}
