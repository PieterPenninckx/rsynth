#[cfg(feature = "stable")]
use syllogism_macro::impl_specialization;
#[cfg(feature = "stable")]
use crate::dev_utilities::compatibility::*;
#[cfg(feature = "stable")]
use crate::middleware::frame_counter::FrameCounter;
#[cfg(feature = "stable")]
impl_specialization!(
    trait NotInCrateRsynth;
    macro macro_for_rsynth;

    type FrameCounter;
);

/// Same as the Borrow trait from `std`, but without the default impls.
pub trait TransparentContext<T> {
    fn get(&self) -> &T;
}

pub trait TransparentContextMut<T> {
    fn get_mut(&mut self) -> &mut T;
}