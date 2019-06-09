#[cfg(feature = "stable")]
use crate::dev_utilities::compatibility::*;
#[cfg(feature = "stable")]
use crate::middleware::frame_counter::FrameCounter;
#[cfg(feature = "stable")]
use syllogism_macro::impl_specialization;
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

// Note: we cannot write a more generic implementation in the following style:
// pub struct GenericContextWrapper<'e, 'c, E, C> {
//      extra: &'e mut E,
//      child_context: &'c mut C,
// }
// because the compiler doesn't know that `E` does not implement `IsNot<E>`,
// so we would get into trouble with specialization.
#[cfg(feature = "stable")]
#[macro_export]
macro_rules! wrap_context{
    ($type_name:ty, $wrapper_name:ident) => {
        pub struct $wrapper_name<'a, 'c, C> {
            aspect: &'a mut $type_name,
            child_context: &'c mut C,
        }

        impl<'a, 'c, C> TransparentContext<$type_name> for $wrapper_name<'a, 'c, C> {
            fn get(&self) -> &$type_name {
                self.aspect
            }
        }

        impl<'a, 'c, C, T> TransparentContext<T> for $wrapper_name<'a, 'c, C>
        where
            C: TransparentContext<T>,
            T: IsNot<$type_name>,
        {
            fn get(&self) -> &T {
                (*self.child_context).get()
            }
        }

        impl<'a, 'c, C> TransparentContextMut<$type_name> for $wrapper_name<'a, 'c, C> {
            fn get_mut(&mut self) -> &mut $type_name {
                self.aspect
            }
        }

        impl<'a, 'c, C, T> TransparentContextMut<T> for $wrapper_name<'a, 'c, C>
        where
            C: TransparentContextMut<T>,
            T: IsNot<$type_name>,
        {
            fn get_mut(&mut self) -> &mut T {
                self.child_context.get_mut()
            }
        }
    }
}
