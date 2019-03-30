pub trait Specialize<T> : Sized {
    fn can_specialize(&self) -> bool {
        false
    }
    /// Must return `Some` if and only if `can_specialize` returns `true`.
    fn specialize(self) -> Option<T> {
        None
    }
}
