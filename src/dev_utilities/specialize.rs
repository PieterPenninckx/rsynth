pub trait Specialize<T> : Sized {
    fn can_specialize(&self) -> bool {
        false
    }
    fn specialize(self) -> Option<T> {
        None
    }
}
