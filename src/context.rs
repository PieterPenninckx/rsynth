//! Traits and macros for working with context.
//!
//! TODO: Describe how the "`With*`" traits work.
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

/// Same as the `BorrowMut` trait from `std`, but without the default impls.
pub trait TransparentContext<T> {
    fn get(&mut self) -> &mut T;
}

// Note: in the case of stable Rust, without the specialization feature,
// we cannot write a more generic implementation in the following style:
// ```
// pub struct GenericContextWrapper<'e, 'c, E, C> {
//      extra: &'e mut E,
//      child_context: &'c mut C,
// }
// ```
// because the compiler doesn't know that `E` does not implement `IsNot<E>`,
// so we would get into trouble with specialization.
//
// TODO: Extend so that type_name accepts a type parameter (to be added as an extra parameter
// to the macro).
// 
/// The generated type implements `TransparentContext<$type_name>` by
/// returning the field `aspect` and `TransparentContext<T>` for "any other"
/// type `T` for which the child context implements `Transparentcontext<T>`
/// by delegating it to the child context.
///
/// The generated type has the following two fields
/// * `aspect`, which is a reference as mutable to the given type, and
/// * `child_context`, which is a reference as mutable to the "child context".
///
/// The generated type has also a non-public `new` function as follows:
/// ```ignore
/// fn new(aspect: &'a mut $type_name, child_context: &'c mut C) -> Self;
/// ```
#[cfg(feature = "stable")]
#[macro_export]
macro_rules! wrap_context {
    ($type_name:ty, $wrapper_name:ident) => {
        pub struct $wrapper_name<'a, 'c, C> {
            aspect: &'a mut $type_name,
            child_context: &'c mut C,
        }

        impl<'a, 'c, C> $wrapper_name<'a, 'c, C> {
            fn new(aspect: &'a mut $type_name, child_context: &'c mut C) -> Self {
                Self {
                    aspect,
                    child_context,
                }
            }
        }

        impl<'a, 'c, C> TransparentContext<$type_name> for $wrapper_name<'a, 'c, C> {
            fn get(&mut self) -> &mut $type_name {
                self.aspect
            }
        }

        impl<'a, 'c, C, T> TransparentContext<T> for $wrapper_name<'a, 'c, C>
        where
            C: TransparentContext<T>,
            T: IsNot<$type_name>,
        {
            fn get(&mut self) -> &mut T {
                self.child_context.get()
            }
        }
    };
}

// Thanks to Lymia for this trick.
// For more info, see
// https://github.com/rust-lang/rust/issues/31844#issuecomment-397650553
#[cfg(not(feature = "stable"))]
#[doc(hidden)]
pub trait UniversalTransparentContext<T> {
    fn get(&mut self) -> &mut T;
}

#[cfg(not(feature = "stable"))]
#[doc(hidden)]
pub trait GenericOrSpecial<T> {}

#[cfg(not(feature = "stable"))]
pub struct ContextWrapper<'a, 'c, A, C> {
    aspect: &'a mut A,
    child_context: &'c mut C,
}

#[cfg(not(feature = "stable"))]
impl<'a, 'c, A, C> ContextWrapper<'a, 'c, A, C> {
    pub fn new(aspect: &'a mut A, child_context: &'c mut C) -> Self {
        ContextWrapper {
            aspect,
            child_context,
        }
    }
}

#[cfg(not(feature = "stable"))]
impl<'a, 'c, A, C, T> UniversalTransparentContext<T> for ContextWrapper<'a, 'c, A, C> {
    default fn get(&mut self) -> &mut T {
        unreachable!();
    }
}

#[cfg(not(feature = "stable"))]
impl<'a, 'c, A, C, T> UniversalTransparentContext<T> for ContextWrapper<'a, 'c, A, C>
where
    C: TransparentContext<T>,
{
    fn get(&mut self) -> &mut T {
        self.child_context.get()
    }
}

#[cfg(not(feature = "stable"))]
impl<'a, 'c, A, C, T> GenericOrSpecial<T> for ContextWrapper<'a, 'c, A, C> where
    C: TransparentContext<T>
{
}

#[cfg(not(feature = "stable"))]
impl<'a, 'c, A, C> GenericOrSpecial<A> for ContextWrapper<'a, 'c, A, C> {}

#[cfg(not(feature = "stable"))]
impl<'a, 'c, A, C, T> TransparentContext<T> for ContextWrapper<'a, 'c, A, C>
where
    ContextWrapper<'a, 'c, A, C>: GenericOrSpecial<T>,
{
    default fn get(&mut self) -> &mut T {
        <Self as UniversalTransparentContext<T>>::get(self)
    }
}

#[cfg(not(feature = "stable"))]
impl<'a, 'c, A, C> TransparentContext<A> for ContextWrapper<'a, 'c, A, C>
where
    ContextWrapper<'a, 'c, A, C>: GenericOrSpecial<A>,
{
    fn get(&mut self) -> &mut A {
        self.aspect
    }
}

/// A macro that generates a type with the given name.
/// The generated type implements `TransparentContext<$type_name>` by
/// returning the field `aspect` and `TransparentContext<T>` for "any other"
/// type `T` for which the child context implements `Transparentcontext<T>`
/// by delegating it to the child context.
///
/// The generated type has the following two fields
/// * `aspect`, which is a reference as mutable to the given type, and
/// * `child_context`, which is a reference as mutable to the "child context".
///
/// The generated type has also a non-public `new` function as follows:
/// ```ignore
/// fn new(aspect: &'a mut $type_name, child_context: &'c mut C) -> Self;
/// ```
#[cfg(not(feature = "stable"))]
#[macro_export]
macro_rules! wrap_context {
    ($type_name:ty, $wrapper_name:ident) => {
        type $wrapper_name<'a, 'c, C> = $crate::context::ContextWrapper<'a, 'c, $type_name, C>;
    };
}
