/// Trait to implement downcasting.

pub trait DowncastCheck<T> {
    fn can_downcast(&self) -> bool;
}

pub trait Downcast<T>: DowncastCheck<T> {
    fn downcast(self) -> Option<T>;
}

pub trait DowncastRef<T>: DowncastCheck<T> {
    fn downcast_ref(&self) -> Option<&T>;
}

pub trait DowncastMut<T>: DowncastCheck<T> {
    fn downcast_mut(&mut self) -> Option<&mut T>;
}

#[macro_export]
macro_rules! impl_downcast {
    ($l:tt, $t: ty) => {
        impl<$l> DowncastCheck<$t> for $t {
            fn can_downcast(&self) -> bool {
                true
            }
        }

        impl<$l, T> DowncastCheck<T> for $t where T: IsNot<$t> {
            fn can_downcast(&self) -> bool {
                false
            }
        }

        impl<$l> Downcast<$t> for $t {
            fn downcast(self) -> Option<$t> {
                Some(self)
            }
        }

        impl<$l, T> Downcast<T> for $t where T: IsNot<$t> {
            fn downcast(self) -> Option<T> {
                None
            }
        }

        impl<$l> DowncastRef<$t> for $t {
            fn downcast_ref(&self) -> Option<&$t> {
                Some(self)
            }
        }

        impl<$l, T> DowncastRef<T> for $t where T: IsNot<$t> {
            fn downcast_ref(&self) -> Option<&T> {
                None
            }
        }

        impl<$l> DowncastMut<$t> for $t {
            fn downcast_mut(&mut self) -> Option<&mut $t> {
                Some(self)
            }
        }

        impl<$l, T> DowncastMut<T> for $t where T: IsNot<$t> {
            fn downcast_mut(&mut self) -> Option<&mut T> {
                None
            }
        }
    };
    ($t:ty) => {
        impl DowncastCheck<$t> for $t {
            fn can_downcast(&self) -> bool {
                true
            }
        }

        impl<T> DowncastCheck<T> for $t where T: IsNot<$t> {
            fn can_downcast(&self) -> bool {
                false
            }
        }

        impl Downcast<$t> for $t {
            fn downcast(self) -> Option<$t> {
                Some(self)
            }
        }

        impl<T> Downcast<T> for $t where T: IsNot<$t> {
            fn downcast(self) -> Option<T> {
                None
            }
        }

        impl DowncastRef<$t> for $t {
            fn downcast_ref(&self) -> Option<&$t> {
                Some(self)
            }
        }

        impl<T> DowncastRef<T> for $t where T: IsNot<$t> {
            fn downcast_ref(&self) -> Option<&T> {
                None
            }
        }

        impl DowncastMut<$t> for $t {
            fn downcast_mut(&mut self) -> Option<&mut $t> {
                Some(self)
            }
        }

        impl<T> DowncastMut<T> for $t where T: IsNot<$t> {
            fn downcast_mut(&mut self) -> Option<&mut T> {
                None
            }
        }
    }
}

