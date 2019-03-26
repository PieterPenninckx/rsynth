use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;
use std::ops::DerefMut;

macro_rules! guards_borrow_field_not_initialised_with_some_value_error {
    ($VecGuard:ident) => {
        concat!(
            "`",
            stringify!($VecGuard),
            "`'s field `borrow` should be initialized with `Some<Vec>`"
        )
    };
}

macro_rules! vec_storage {
    ($VecStorage:ident, $T:ident, $VecGuard:ident, $b:lifetime, $amp_b_T:ty, $amp_T:ty) => {
        #[derive(Debug)]
        pub struct $VecStorage<$T>
        where
            $T: ?Sized,
        {
            // We use `usize` here, because `*mut &$T` requires a lifetime, which we
            // cannot specify here.
            // Note: because of this, $VecStorage implements `Send` and `Sync`.
            ptr: usize,
            capacity: usize,
            // The borrow system already ensures that there cannot be two `VecGuard`'s of
            // the same `VecStorage`, but when a `VecGuard` is "mem::forgotten", it cannot
            // cleanup, so we use this field to ensure that no new `VecGuard` can be created
            // if the previous one is "mem::forgotten".
            is_locked: bool,
            phantom: PhantomData<$T>,
        }

        pub struct $VecGuard<'s, $b, $T>
        where
            $T: ?Sized,
        {
            storage: &'s mut $VecStorage<$T>,
            // We use an `Option` here because `drop` is always called recursively,
            // see https://doc.rust-lang.org/nomicon/destructors.html
            borrow: Option<Vec<$amp_b_T>>,
        }

        impl<'s, $b, $T> Deref for $VecGuard<'s, 'b, $T>
        where
            $T: ?Sized,
        {
            type Target = Vec<$amp_b_T>;

            fn deref(&self) -> &Vec<$amp_b_T> {
                self.borrow.as_ref().expect(
                    guards_borrow_field_not_initialised_with_some_value_error!($VecGuard),
                )
            }
        }

        impl<'s, $b, $T> DerefMut for $VecGuard<'s, $b, $T>
        where
            $T: ?Sized,
        {
            fn deref_mut(&mut self) -> &mut Vec<$amp_b_T> {
                self.borrow.as_mut().expect(
                    guards_borrow_field_not_initialised_with_some_value_error!($VecGuard),
                )
            }
        }

        impl<'s, $b, $T> Drop for $VecGuard<'s, $b, $T>
        where
            $T: ?Sized,
        {
            fn drop(&mut self) {
                let mut v = self.borrow.take().expect(
                    guards_borrow_field_not_initialised_with_some_value_error!($VecGuard),
                );
                v.clear();
                self.storage.ptr = v.as_mut_ptr() as usize;
                debug_assert_eq!(v.len(), 0);
                self.storage.capacity = v.capacity();

                mem::forget(v);

                self.storage.is_locked = false;
            }
        }

        impl<$T> $VecStorage<$T>
        where
            $T: ?Sized,
        {
            pub fn with_capacity(capacity: usize) -> Self {
                let mut vector: Vec<$amp_T> = Vec::with_capacity(capacity);
                debug_assert_eq!(vector.len(), 0);
                let result = Self {
                    is_locked: false,
                    ptr: vector.as_mut_ptr() as usize,
                    capacity: vector.capacity(),
                    phantom: PhantomData,
                };
                mem::forget(vector);
                result
            }

            /// Creates a new $VecGuard using the memory allocated by `self`.
            /// This $VecGuard will automatically clear the vector when it goes
            /// out of scope.
            /// # Panics
            /// Panics if `mem::forget()` was called on a `BorrowGuard`.
            pub fn vec_guard<'s, $b>(&'s mut self) -> $VecGuard<'s, $b, $T> {
                // If `mem::forget()` was called on the guard, then
                // the `drop()` on the guard did not run and
                // the ptr and the capacity of the underlying vector may not be
                // correct anymore.
                // It is then undefined behaviour to use `Vec::from_raw_parts`.
                // Hence this check.
                if self.is_locked {
                    panic!(concat!(
                        "`",
                        stringify!($VecStorage),
                        "` has been locked. Probably `mem::forget()` was called on a `",
                        stringify!($VecGuardname),
                        "`"
                    ))
                }
                self.is_locked = true;

                let vector;
                #[allow(unused_unsafe)]
                unsafe {
                    vector = Vec::from_raw_parts(self.ptr as *mut $amp_T, 0, self.capacity)
                }
                $VecGuard {
                    borrow: Some(vector),
                    storage: self,
                }
            }
        }

        impl<$T> Drop for $VecStorage<$T>
        where
            $T: ?Sized,
        {
            fn drop(&mut self) {
                if !self.is_locked {
                    unsafe {
                        mem::drop(Vec::from_raw_parts(
                            self.ptr as *mut $amp_T,
                            0,
                            self.capacity,
                        ));
                    }
                } else {
                    // If `mem::forget()` was called on a guard, then
                    // the `drop()` on the guard did not run and
                    // the ptr and the capacity of the underlying vector may not be
                    // correct anymore.
                    // It is probably not a good idea to panic inside the `drop()` function,
                    // so let's just leak some memory (`mem::forget()` was called after all.)
                    // We do nothing in this `else` branch.
                }
            }
        }
    };
}
vec_storage!(VecStorage, T, VecGuard, 'b, &'b T, &T);
vec_storage!(VecStorageMut, T, VecGuardMut, 'b, &'b mut T, &mut T);
