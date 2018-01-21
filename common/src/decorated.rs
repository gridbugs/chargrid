/// Represents a particular view `V`, decorated by a decorator `D`.
pub struct Decorated<'a, 'b, V: 'a, D: 'b> {
    pub view: &'a mut V,
    pub decorator: &'b D,
}
impl<'a, 'b, V, D> Decorated<'a, 'b, V, D> {
    pub fn new(view: &'a mut V, decorator: &'b D) -> Self {
        Self { view, decorator }
    }
}
