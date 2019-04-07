pub trait NotInApplication {}
pub trait NotInCrateRsynth {}

#[macro_export]
macro_rules! not_in_application {
    () => {
        $crate::dev-utilities::compatibility::NotInApplication
    }
}

macro_rules! impl_traits_for_rsynth {
    ($($x:tt)*) => {
        impl_traits!(($crate::dev_utilities::compatibility::NotInApplication,), $($x)*);
    }
}
