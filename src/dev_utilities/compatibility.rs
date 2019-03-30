pub trait NotInApplication {}
pub trait NotInCrateRsynth {}

macro_rules! traits_for_rsynth {
    () => {
        (NotInApplication,)
    }
}

/// Macro to implement a given list of traits for a given type.
/// Examples:
/// ```
/// trait T1 {}
/// trait T2 {}
/// struct S<'a, T> {data: &'a T}
/// impl_traits!((T1, T2,), impl<'a, T> trait for S<'a, T>);
/// ```
#[macro_export]
macro_rules! impl_traits {
    (($($traits:tt,)*), impl<$head:tt $(,$tail:tt)*> trait for $t:ty) => {
         impl_traits!(@impl_traits ($($traits,)*) @ $t , $head @ ($($tail,)*));
    };
    (($($traits:tt,)*), impl trait for $t:ty) => {
         $(impl $traits for $t {})*
    };
    (@impl_traits ($($traits:tt,)*) @ $t:ty , $head:tt @ $tuple:tt) => {
        $(impl_traits!(@impl_one_trait $traits @ $t , $head @ $tuple);)*
    };
    (@impl_one_trait $one_trait:tt @ $t:ty , $head:tt @ ($($tail:tt)*)) => {
        impl<$head $(,$tail)*> $one_trait for $t {}
    }
}