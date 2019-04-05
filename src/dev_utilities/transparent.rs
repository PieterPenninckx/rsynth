/// A trait for defining middleware that can work with different back-ends.
///
/// Suppose `M` is middleware and a plugin `P` implements the `Plugin` trait and
/// another backend-specific trait. Then a blanket impl defined for the backend
/// will ensure that `M<P>` will also implement the backend-specific trait if
/// `M<P>' implements `Transparent<Inner=P>`
pub trait Transparent {
    type Inner;
    fn get(&self) -> &Self::Inner;
    fn get_mut(&mut self) -> &mut Self::Inner;
}