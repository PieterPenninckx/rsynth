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
macro_rules! wrap_context {
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
    };
}

#[cfg(not(feature = "stable"))]
macro_rules! wrap_context {
    ($type_name:ty, $wrapper_name:ident) => {
        pub struct $wrapper_name<'a, 'c, C> {
            aspect: &'a mut $type_name,
            child_context: &'c mut C,
        }

        // Thanks to Lymia for this trick.
        // For more info, see
        // https://github.com/rust-lang/rust/issues/31844#issuecomment-397650553
        trait UniversalTransparentContext<T> {
            fn get(&self) -> &T;
        }

        trait UniversalTransparentContextMut<T> {
            fn get_mut(&mut self) -> &mut T;
        }

        impl<'a, 'c, C, T> UniversalTransparentContext<T> for $wrapper_name<'a, 'c, C> {
            default fn get(&self) -> &T {
                unreachable!();
            }
        }

        impl<'a, 'c, C, T> UniversalTransparentContext<T> for $wrapper_name<'a, 'c, C>
        where
            C: TransparentContext<T>,
        {
            fn get(&self) -> &T {
                self.child_context.get()
            }
        }

        impl<'a, 'c, C, T> UniversalTransparentContextMut<T> for $wrapper_name<'a, 'c, C> {
            default fn get_mut(&mut self) -> &mut T {
                unreachable!();
            }
        }

        impl<'a, 'c, C, T> UniversalTransparentContextMut<T> for $wrapper_name<'a, 'c, C>
        where
            C: TransparentContextMut<T>,
        {
            fn get_mut(&mut self) -> &mut T {
                self.child_context.get_mut()
            }
        }

        trait GenericOrSpecial<T> {}

        impl<'a, 'c, C, T> GenericOrSpecial<T> for $wrapper_name<'a, 'c, C> where
            C: TransparentContext<T>
        {
        }

        impl<'a, 'c, C> GenericOrSpecial<$type_name> for $wrapper_name<'a, 'c, C> {}

        impl<'a, 'c, C, T> TransparentContext<T> for $wrapper_name<'a, 'c, C>
        where
            $wrapper_name<'a, 'c, C>: GenericOrSpecial<T>,
        {
            default fn get(&self) -> &T {
                <Self as UniversalTransparentContext<T>>::get(self)
            }
        }

        impl<'a, 'c, C> TransparentContext<$type_name> for $wrapper_name<'a, 'c, C>
        where
            $wrapper_name<'a, 'c, C>: GenericOrSpecial<$type_name>,
        {
            fn get(&self) -> &FrameCounter {
                self.aspect
            }
        }

        impl<'a, 'c, C, T> TransparentContextMut<T> for $wrapper_name<'a, 'c, C>
        where
            $wrapper_name<'a, 'c, C>: GenericOrSpecial<T>,
        {
            default fn get_mut(&mut self) -> &mut T {
                <Self as UniversalTransparentContextMut<T>>::get_mut(self)
            }
        }

        impl<'a, 'c, C> TransparentContextMut<$type_name> for $wrapper_name<'a, 'c, C>
        where
            $wrapper_name<'a, 'c, C>: GenericOrSpecial<$type_name>,
        {
            fn get_mut(&mut self) -> &mut $type_name {
                self.aspect
            }
        }
    };
}
