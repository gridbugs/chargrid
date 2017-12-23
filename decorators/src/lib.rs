extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate prototty_traits;
extern crate prototty_defaults;

pub struct Decorated<'a, 'b, V: 'a, D: 'b> {
    pub view: &'a V,
    pub decorator: &'b D,
}
impl<'a, 'b, V, D> Decorated<'a, 'b, V, D> {
    pub fn new(view: &'a V, decorator: &'b D) -> Self {
        Self { view, decorator }
    }
}

mod border;
pub use self::border::Border;
