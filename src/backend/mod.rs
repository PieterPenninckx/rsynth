//! Defines the JACK backend and the VST backend.
use asprim::AsPrim;
use num_traits::Float;

#[cfg(feature = "jack-backend")]
pub mod jack_backend;
pub mod vst_backend;

/// The trait that all plugins need to implement.
/// The type parameter `E` represents the type of events the plugin supports.
pub trait Plugin<E> {
    /// The name of the plugin.
    const NAME: &'static str;

    /// The maximum number of audio inputs.
    const MAX_NUMBER_OF_AUDIO_INPUTS: usize;

    /// The maximum number of audio outputs.
    const MAX_NUMBER_OF_AUDIO_OUTPUTS: usize;

    /// The name of the audio input with the given index.
    /// Note: you may not provide an empty string to the Jack backend.
    fn audio_input_name(index: usize) -> String;

    /// The name of the audio output with the given index.
    /// Note: you may not provide an empty string to the Jack backend.
    fn audio_output_name(index: usize) -> String;

    /// Called when the sample-rate changes.
    /// The backend should ensure that this function is called before
    /// any other.
    fn set_sample_rate(&mut self, sample_rate: f64);

    /// This function is the core of the plugin.
    /// It is called repeatedly for subsequent buffers.
    /// The length of `inputs` is guaranteed to be smaller than or equal to
    /// `Self::MAX_NUMBER_OF_AUDIO_INPUTS`.
    /// The length of `outputs` is guaranteed to be smaller than or equal to
    /// `Self::MAX_NUMBER_OF_AUDIO_OUTPUTS`.
    /// The lengths of all elements of `inputs` and the lengths of all elements of `outputs`
    /// are all guaranteed to equal to each other.
    /// This shared length can however be different for subsequent calls to `render_buffer`.
    //Right now, the `render_buffer` function is generic over floats. How do we specialize
    //  if we want to use SIMD?
    fn render_buffer<F>(&mut self, inputs: &[&[F]], outputs: &mut [&mut [F]])
    where
        F: Float + AsPrim;

    /// This function is called for each event.
    fn handle_event(&mut self, event: &E);
}

/// Utilities to handle both polyphonic and monophonic plugins.
pub mod output_mode {
    use num_traits::Float;

    /// Defines a method to set an output sample.
    pub trait OutputMode: Default {
        fn set<F>(f: &mut F, value: F)
        where
            F: Float;
    }

    /// Output by adding the sample to what is already in the output.
    /// Useful in a polyphonic context.
    #[derive(Default)]
    pub struct Additive {}

    impl OutputMode for Additive {
        #[inline(always)]
        fn set<F>(f: &mut F, value: F)
        where
            F: Float,
        {
            *f = *f + value;
        }
    }

    /// Output by replacing what is already in the output by the given value.
    /// Useful in a monophonic context.
    #[derive(Default)]
    pub struct Substitution {}

    impl OutputMode for Substitution {
        #[inline(always)]
        fn set<F>(f: &mut F, value: F)
        where
            F: Float,
        {
            *f = value;
        }
    }
}

pub struct RawMidiEvent<'a> {
    pub data: &'a [u8],
}

pub enum Event<T, U> {
    Timed { samples: u32, event: T },
    UnTimed(U),
}

/// A trait for defining middleware that can work with different back-ends.
///
/// Suppose `M` is middleware and a plugin `P` implements the `Plugin` trait and
/// another backend-specific trait. Then a blanket impl defined for the backend
/// will ensure that `M<P>` will also implement the backend-specific trait if
/// `M<P>' implements `Transparent<Inner=P>`
pub trait Transparent {
    type Inner;
    fn get(&self) -> &Self::Inner;
    fn get_mut(&mut self) -> &mut Self::Inner;
}

/// Utilities to be used when developing backends.
/// # `VecStorage` and `VecStorageMut`
/// Struct to be able to re-use the storage of a vector
/// for borrowing values with different lifetimes.
///
/// ## Examples
/// The following code does not compile:
/// ```ignore
/// let mut v = Vec::with_capacity(2);
/// {
///     let x = 1; let y = 2;
///     v.push(&x);
///     v.push(&y);
///     v.clear(); // We stop borrowing here, but the compiler doesn't know that.
/// }
/// {
///     let a = 1; let b = 2;
///     v.push(&a);
///     v.push(&b);
///     v.clear(); // We stop borrowing here, but the compiler doesn't know that.
/// }
/// ```
///
/// You can use `VecStorage` to solve this problem:
/// ```
/// use rsynth::backend::utilities::VecStorage;
/// let mut v = VecStorage::with_capacity(2);
/// {
///     let x = 1; let y = 2;
///     let mut guard = v.vec_guard();
///     // Now guard behaves like a vector.
///     guard.push(&x); // No memory allocation here, we use the memory allocated in `v`.
///     guard.push(&y);
///     // If we were going to push more items on the guard, we would allocate memory.
///     // When guard goes out of scope, it is cleared.
/// }
/// {
///     let a = 1; let b = 2;
///     let mut guard = v.vec_guard();
///     // Now guard behaves like a vector.
///     // The memory from the previous run has been cleared ...
///     assert_eq!(guard.len(), 0);
///     guard.push(&a);
///     guard.push(&b);
/// }
/// ```
///
/// The `VecStorage` re-uses the same memory each time:
/// ```
/// use rsynth::backend::utilities::VecStorage;
/// let mut v = VecStorage::with_capacity(2);
/// let capacity;
/// {
///     let x = 1; let y = 2; let z = 3;
///     let mut guard = v.vec_guard();
///     guard.push(&x); // No memory allocation here, we use the memory allocated in `v`.
///     guard.push(&y);
///     // Let's push some more items on the guard and allocate memory:
///     guard.push(&z);
///     capacity = guard.capacity();
///     assert!(capacity > 2);
/// }
/// {
///     let mut guard = v.vec_guard();
///     // The memory from the previous run has been cleared ...
///     assert_eq!(guard.len(), 0);
///     // ... but the capacity is kept:
///     assert_eq!(capacity, guard.capacity());
/// }
/// ```
///
/// `VecStorage<T>` allocates memory just like `Vec<&T>`,
/// but it does not borrow anything.
/// You can create a `VecGuard` with the `vec_guard` method.
/// The `VecGuard` uses the memory from the `VecStorage` and can temporarily
/// be used just like a `Vec<&T>`
/// (i.e.: it implements `Deref<Target=Vec<&T>>` and `DerefMut<Target=Vec<&T>>`)
/// When the `VecGuard` is dropped, the memory "goes back to the `VecStorage`" and
/// can be re-used later on to store references with a different lifetime.
///
/// `VecStorageMut<T>` is similar: it allows you to create a `VecGuardMut`, which
/// can be used just like a `Vec<&mut T>`.
pub mod utilities {
    use std::marker::PhantomData;
    use std::mem;
    use std::ops::Deref;
    use std::ops::DerefMut;

    macro_rules! vec_storage {
        ($VecStorage:ident, $T:ident, $VecGuard:ident, $b:lifetime, $amp_b_T:ty, $amp_T:ty, $VecStorageName:expr, $VecGuardName:expr) => {
        
            /// Re-usable memory for creating a vector of references.
            ///
            /// See the [module-level documentation] for more information.
            ///
            /// [module-level documentation]: ./index.html
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

            /// This can be used as a vector of references.
            ///
            /// See the [module-level documentation] for more information.
            ///
            /// [module-level documentation]: ./index.html
            pub struct $VecGuard<'s, $b, $T>
            where
                $T: ?Sized,
            {
                storage: &'s mut $VecStorage<$T>,
                borrow: Vec<$amp_b_T>,
            }

            impl<'s, $b, $T> Deref for $VecGuard<'s, 'b, $T>
            where
                $T: ?Sized,
            {
                type Target = Vec<$amp_b_T>;

                fn deref(&self) -> &Vec<$amp_b_T> {
                    &self.borrow
                }
            }

            impl<'s, $b, $T> DerefMut for $VecGuard<'s, $b, $T>
            where
                $T: ?Sized,
            {
                fn deref_mut(&mut self) -> &mut Vec<$amp_b_T> {
                    &mut self.borrow
                }
            }

            impl<'s, $b, $T> Drop for $VecGuard<'s, $b, $T>
            where
                $T: ?Sized,
            {
                fn drop(&mut self) {
                    self.borrow.clear();
                    self.storage.ptr = self.borrow.as_mut_ptr() as usize;
                    debug_assert_eq!(self.borrow.len(), 0);
                    self.storage.capacity = self.borrow.capacity();

                    // `drop` is always called recursively,
                    // see https://doc.rust-lang.org/nomicon/destructors.html
                    // So we have to manually drop `self.borrow`.
                    // We cannot simply "move out of borrowed content",
                    // so we swap it with another vector.
                    // Note: `Vec::new()` does not allocate.
                    let mut v = Vec::new();
                    mem::swap(&mut v, &mut self.borrow);
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

                #[doc="Creates a new "]
                #[doc=$VecGuardName]
                #[doc="using the memory allocated by `self`. This `"]
                #[doc=$VecGuardName]
                #[doc="` will automatically clear the vector when it goes out of scope."]
                #[doc="# Panics\n"]
                #[doc="Panics if `mem::forget()` was called on a `"]
                #[doc=$VecGuardName]
                #[doc="` that was created previously on the same `"]
                #[doc=$VecStorageName]
                #[doc="`."]
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
                            $VecStorageName,
                            "` has been locked. Probably `mem::forget()` was called on a `",
                            $VecGuardName,
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
                        borrow: vector,
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
        
        ($VecStorage:ident, $T:ident, $VecGuard:ident, $b:lifetime, $amp_b_T:ty, $amp_T:ty) => {
            vec_storage!($VecStorage, $T, $VecGuard, $b, $amp_b_T, $amp_T, stringify!($VecStorage), stringify!($VecGuard));
        };
    }
    vec_storage!(VecStorage, T, VecGuard, 'b, &'b T, &T);
    vec_storage!(VecStorageMut, T, VecGuardMut, 'b, &'b mut T, &mut T);
    
    #[test]
    #[should_panic(expected="`VecStorage` has been locked. Probably `mem::forget()` was called on a `VecGuard`")]
    fn mem_forgetting_guard_leads_to_panic_with_new_guard() {
        use ::backend::utilities::VecStorage;
        let mut v = VecStorage::with_capacity(2);
        {
            let x = 1;
            let mut guard = v.vec_guard();
            guard.push(&x);
            // You should not do the following:
            mem::forget(guard);
        }
        {
            let guard = v.vec_guard();
        }
    }

    #[test]
    fn mem_forgetting_guard_does_not_lead_to_panic() {
        use ::backend::utilities::VecStorage;
        let mut v = VecStorage::with_capacity(2);
        {
            let x = 1;
            let mut guard = v.vec_guard();
            guard.push(&x);
            // You should not do the following:
            mem::forget(guard);
        }
        // The `VecStorage` is dropped and this should not lead to any problem.
    }

    #[test]
    fn vec_storage_mut_common_use_cases() {
        use ::backend::utilities::VecStorageMut;
        let capacity;
        let mut v = VecStorageMut::with_capacity(2);
        {
            let mut x = 1;
            let mut y = 2;
            let mut z = 3;
            let mut guard = v.vec_guard();
            assert_eq!(guard.capacity(), 2);
            assert_eq!(guard.len(), 0);
            guard.push(&mut x);
            guard.push(&mut y);
            guard.push(&mut z);
            capacity = guard.capacity();
        }
        {
            let mut a = 1;
            let mut b = 2;
            let mut guard = v.vec_guard();
            assert_eq!(guard.len(), 0);
            assert_eq!(capacity, guard.capacity());
            guard.push(&mut a);
            guard.push(&mut b);
        }
    }
}
