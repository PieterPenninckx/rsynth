use num_traits::Float;
use asprim::AsPrim;

pub mod vst_backend;
#[cfg(feature="jack-backend")]
pub mod jack_backend;

pub trait Plugin<E> {
    /// The name of the plugin.
    const NAME: &'static str;

    /// The maximum number of audio inputs.
    const MAX_NUMBER_OF_AUDIO_INPUTS: usize;

    /// The maximum number of audio outputs.
    const MAX_NUMBER_OF_AUDIO_OUTPUTS: usize;

    /// The name of the audio input with the given index.
    /// Note: you may not provide an empty string to the Jack backend
    fn audio_input_name(index: usize) -> String;

    /// The name of the audio output with the given index.
    /// Note: you may not provide an empty string to the Jack backend
    fn audio_output_name(index: usize) -> String;

    /// Called when the sample-rate changes.
    /// TODO: Make sure that this is also called initially.
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
    fn render_buffer<F>(&mut self, inputs: &[&[F]], outputs: &mut[&mut[F]])
        where F: Float + AsPrim;

    /// This function is called for each event.
    fn handle_event(&mut self, event: &E);
}

pub struct RawMidiEvent<'a> {
    pub data: &'a [u8]
}

pub enum Event<T, U> {
    Timed{samples: u32, event: T},
    UnTimed(U)
}

/// A utility trait for defining middleware that can work with different back-ends.
/// Suppose `M` is middleware and a plugin `P` implement the `Plugin` trait and
/// a other backend-specific trait, then a blanket impl defined for the backend
/// will ensure that `M<P>` will also implement the backend-specific trait if
/// `M<P>' implements `Transparent<Inner=P>`
pub trait Transparent {
    type Inner;
    fn get(&self) -> &Self::Inner;
    fn get_mut(&mut self) -> &mut Self::Inner;
}

/// Utilities to be used when developing backends.
pub mod utilities {
    use std::mem;
    use std::ops::Deref;
    use std::ops::DerefMut;
    use std::marker::PhantomData;

    #[derive(Debug)]
    pub struct Hibernation {
        ptr: usize,
        capacity: usize,
        is_locked: bool
    }

    #[derive(Debug)]
    pub struct HibernationMut {
        ptr: usize,
        capacity: usize,
        is_locked: bool
    }

    pub struct BorrowGuard<'h, 'b, T>
    where T: ?Sized
    {
        hibernation: &'h mut Hibernation,
        // We use an `Option` here because the compiler does not (yet)
        // allow us to `mem::forget` our fields, i.e. `drop` is always
        // called recursively, see https://doc.rust-lang.org/nomicon/destructors.html
        borrow: Option<Vec<&'b T>>
    }

    pub struct BorrowGuardMut<'h, 'b, T>
    where T: ?Sized
    {
        hibernation: &'h mut HibernationMut,
        // We use an `Option` here because the compiler does not (yet)
        // allow us to `mem::forget` our fields, i.e. `drop` is always
        // called recursively, see https://doc.rust-lang.org/nomicon/destructors.html
        borrow: Option<Vec<&'b mut T>>
    }

    impl<'h, 'b, T> Deref for BorrowGuard<'h, 'b, T>
    where T : ?Sized
    {
        type Target = Vec<&'b T>;

        fn deref(&self) -> &Vec<&'b T> {
            self.borrow.as_ref().expect("`BorrowGuard` should be constructed with `Some<Vec>`")
        }
    }

    impl<'h, 'b, T> Deref for BorrowGuardMut<'h, 'b, T>
    where T : ?Sized
    {
        type Target = Vec<&'b mut T>;

        fn deref(&self) -> &Vec<&'b mut T> {
            self.borrow.as_ref().expect("`BorrowGuardMut` should be constructed with `Some<Vec>`")
        }
    }

    impl<'h, 'b, T> DerefMut for BorrowGuard<'h, 'b, T>
    where T : ?Sized
    {
        fn deref_mut(&mut self) -> &mut Vec<&'b T> {
            self.borrow.as_mut().expect("`BorrowGuard` should be constructed with `Some<Vec>`")
        }
    }

    impl<'h, 'b, T> DerefMut for BorrowGuardMut<'h, 'b, T>
    where T : ?Sized
    {
        fn deref_mut(&mut self) -> &mut Vec<&'b mut T> {
            self.borrow.as_mut().expect("`BorrowGuardMut` should be constructed with `Some<Vec>`")
        }
    }

    impl<'h, 'b, T> Drop for BorrowGuard<'h, 'b, T>
    where T : ?Sized {
        fn drop(&mut self) {
            let mut v = self.borrow.take().expect("`BorrowGuard` should be constructed with `Some<Vec>`");

            v.clear();
            self.hibernation.ptr = v.as_mut_ptr() as usize;
            debug_assert_eq!(v.len(), 0);
            self.hibernation.capacity = v.capacity();

            mem::forget(v);

            self.hibernation.is_locked = false;
        }
    }

    impl<'h, 'b, T> Drop for BorrowGuardMut<'h, 'b, T>
    where T : ?Sized {
        fn drop(&mut self) {
            let mut v = self.borrow.take().expect("`BorrowGuardMut` should be constructed with `Some<Vec>`");

            v.clear();
            self.hibernation.ptr = v.as_mut_ptr() as usize;
            debug_assert_eq!(v.len(), 0);
            self.hibernation.capacity = v.capacity();

            mem::forget(v);

            self.hibernation.is_locked = false;
        }
    }

    impl Hibernation {
        pub fn new(capacity: usize) -> Self {
            // All references have the same size and alignment.
            let mut vector: Vec<&()> = Vec::with_capacity(capacity);
            debug_assert_eq!(vector.len(), 0);
            let result = Self {
                is_locked: false,
                ptr: vector.as_mut_ptr() as usize,
                capacity: vector.capacity()
            };
            mem::forget(vector);
            result
        }

        /// # Panics
        /// Panics if `mem::forget()` was called on a `BorrowGuard`.
        pub fn borrow_mut<'h, 'b, T>(&'h mut self) -> BorrowGuard<'h, 'b, T>
        where T : ?Sized
        {
            // If `mem::forget()` was called on a `BorrowGuard`, then
            // the `drop()` on the `BorrowGuard` did not run and
            // the ptr and the capacity of the underlying vector may not be
            // correct anymore.
            // It is then undefined behaviour to use `Vec::from_raw_parts`.
            // Hence this check.
            if self.is_locked {
                panic!("`Hibernation` has been locked. Probably `mem::forget()` was called on a `BorrowGuard`.");
            }
            self.is_locked = true;


            let vector;
            #[allow(unused_unsafe)]
            unsafe {
                vector = Vec::from_raw_parts(self.ptr as *mut &T, 0, self.capacity)
            }
            BorrowGuard {
                borrow: Some(vector),
                hibernation: self
            }
        }
    }

    impl HibernationMut {
        pub fn new(capacity: usize) -> Self {
            // All references have the same size and alignment.
            let mut vector: Vec<&()> = Vec::with_capacity(capacity);
            debug_assert_eq!(vector.len(), 0);
            let result = Self {
                is_locked: false,
                ptr: vector.as_mut_ptr() as usize,
                capacity: vector.capacity()
            };
            mem::forget(vector);
            result
        }

        /// # Panics
        /// Panics if `mem::forget()` was called on a `BorrowGuardMut`.
        pub fn borrow_mut<'h, 'b, T>(&'h mut self) -> BorrowGuardMut<'h, 'b, T>
        where T : ?Sized
        {
            // If `mem::forget()` was called on a `BorrowGuard`, then
            // the `drop()` on the `BorrowGuardMut` did not run and
            // the ptr and the capacity of the underlying vector may not be
            // correct anymore.
            // It is then undefined behaviour to use `Vec::from_raw_parts`.
            // Hence this check.
            if self.is_locked {
                panic!("`Hibernation` has been locked. Probably `mem::forget()` was called on a `BorrowGuardMut`.");
            }
            self.is_locked = true;


            let vector;
            #[allow(unused_unsafe)]
                unsafe {
                vector = Vec::from_raw_parts(self.ptr as *mut &mut T, 0, self.capacity)
            }
            BorrowGuardMut {
                borrow: Some(vector),
                hibernation: self
            }
        }
    }

    impl Drop for Hibernation {
        fn drop(&mut self) {
            if ! self.is_locked {
                unsafe {
                    mem::drop(Vec::from_raw_parts(self.ptr as *mut &(), 0, self.capacity));
                }
            } else {
                // If `mem::forget()` was called on a `BorrowGuard`, then
                // the `drop()` on the `BorrowGuard` did not run and
                // the ptr and the capacity of the underlying vector may not be
                // correct anymore.
                // It is probably not a good idea to panic inside the `drop()` function,
                // so let's just leak some memory (`mem::forget()` was called after all.)
                // We do nothing in this `else` branch.
            }
        }
    }

    impl Drop for HibernationMut {
        fn drop(&mut self) {
            if ! self.is_locked {
                unsafe {
                    mem::drop(Vec::from_raw_parts(self.ptr as *mut &(), 0, self.capacity));
                }
            } else {
                // If `mem::forget()` was called on a `BorrowGuardMut`, then
                // the `drop()` on the `BorrowGuard` did not run and
                // the ptr and the capacity of the underlying vector may not be
                // correct anymore.
                // It is probably not a good idea to panic inside the `drop()` function,
                // so let's just leak some memory (`mem::forget()` was called after all.)
                // We do nothing in this `else` branch.
            }
        }
    }
}

