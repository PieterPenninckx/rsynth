//! Audio buffers.

pub trait DelegateHandling<P, D> {
    type Output;
    fn delegate_handling(&mut self, p: &mut P, d: D) -> Self::Output;
}
