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

    macro_rules! guards_borrow_field_not_initialised_with_some_value_error {
        ($Guard:ident) => {
            concat!(
                "`",
                stringify!($Guard),
                "`'s field `borrow` should be initialized with `Some<Vec>`"
            )
        }
    }

    macro_rules! hibernation {
        ($Hibernation:ident, $T:ident, $Guard:ident, $b:lifetime, $amp_b_T:ty, $amp_T:ty) => {

            #[derive(Debug)]
            pub struct $Hibernation<$T>
            where $T: ?Sized
            {
                ptr: usize,
                capacity: usize,
                is_locked: bool,
                phantom: PhantomData<$T>
            }

            pub struct $Guard<'h, $b, $T>
            where $T: ?Sized
            {
                hibernation: &'h mut $Hibernation<$T>,
                // We use an `Option` here because `drop` is always called recursively,
                // see https://doc.rust-lang.org/nomicon/destructors.html
                borrow: Option<Vec<$amp_b_T>>
            }

            impl<'h, $b, $T> Deref for $Guard<'h, 'b, $T>
            where $T : ?Sized
            {
                type Target = Vec<$amp_b_T>;

                fn deref(&self) -> &Vec<$amp_b_T> {
                    self.borrow.as_ref()
                        .expect(guards_borrow_field_not_initialised_with_some_value_error!($Guard))
                }
            }

            impl<'h, $b, $T> DerefMut
            for $Guard<'h, $b, $T>
            where $T : ?Sized
            {
                fn deref_mut(&mut self) -> &mut Vec<$amp_b_T> {
                    self.borrow.as_mut()
                        .expect(guards_borrow_field_not_initialised_with_some_value_error!($Guard))
                }
            }

            impl<'h, $b, $T> Drop for $Guard<'h, $b, $T>
            where $T : ?Sized {
                fn drop(&mut self) {
                    let mut v = self.borrow.take()
                        .expect(guards_borrow_field_not_initialised_with_some_value_error!($Guard));
                    v.clear();
                    self.hibernation.ptr = v.as_mut_ptr() as usize;
                    debug_assert_eq!(v.len(), 0);
                    self.hibernation.capacity = v.capacity();

                    mem::forget(v);

                    self.hibernation.is_locked = false;
                }
            }

            impl<$T> $Hibernation<$T>
            where $T : ?Sized
            {
                pub fn new(capacity: usize) -> Self {
                    let mut vector: Vec<$amp_T> = Vec::with_capacity(capacity);
                    debug_assert_eq!(vector.len(), 0);
                    let result = Self {
                        is_locked: false,
                        ptr: vector.as_mut_ptr() as usize,
                        capacity: vector.capacity(),
                        phantom: PhantomData
                    };
                    mem::forget(vector);
                    result
                }

                /// # Panics
                /// Panics if `mem::forget()` was called on a `BorrowGuard`.
                pub fn borrow_mut<'h, $b>(&'h mut self)
                    -> $Guard<'h, $b, $T>
                {
                    // If `mem::forget()` was called on the guard, then
                    // the `drop()` on the guard did not run and
                    // the ptr and the capacity of the underlying vector may not be
                    // correct anymore.
                    // It is then undefined behaviour to use `Vec::from_raw_parts`.
                    // Hence this check.
                    if self.is_locked {
                        panic!(
                            concat!(
                                "`",
                                stringify!($Hibernation),
                                "` has been locked. Probably `mem::forget()` was called on a `",
                                stringify!($guardname),
                                 "`"
                            )
                        )
                    }
                    self.is_locked = true;


                    let vector;
                    #[allow(unused_unsafe)]
                    unsafe {
                        vector = Vec::from_raw_parts(self.ptr as *mut $amp_T, 0, self.capacity)
                    }
                    $Guard {
                        borrow: Some(vector),
                        hibernation: self
                    }
                }
            }

            impl<$T> Drop for $Hibernation<$T>
            where $T : ?Sized
            {
                fn drop(&mut self) {
                    if ! self.is_locked {
                        unsafe {
                            mem::drop(Vec::from_raw_parts(self.ptr as *mut $amp_T, 0, self.capacity));
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
        }
    }
    hibernation!(Hibernation, T, BorrowGuard, 'b, &'b T, &T);
    hibernation!(HibernationMut, T, BorrowGuardMut, 'b, &'b mut T, &mut T);
}