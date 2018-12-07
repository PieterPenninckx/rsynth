use num_traits::Float;
use asprim::AsPrim;
use std::mem;
use std::process;

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

#[derive(Debug)]
pub struct Hibernation {
    ptr: usize,
    length: usize,
    capacity: usize
}

impl Hibernation {
    fn new<T>(capacity: usize) -> Self {
        let mut vector : Vec<T> = Vec::with_capacity(capacity);
        let result = Self {
            ptr: vector.as_mut_ptr() as usize,
            length: vector.len(),
            capacity: vector.capacity()
        };
        mem::forget(vector);
        result
    }

    /// Must be called with the same data-type as the `new` function was called with.
    unsafe fn wake_up<T>(&self) -> Vec<T> {
        #[allow(unused_unsafe)]
        unsafe {
            Vec::from_raw_parts(self.ptr as *mut T, self.length, self.capacity)
        }
    }

    /// Must be called with the result from `wake_up`.
    fn hibernate<T>(&mut self, mut vector: Vec<T>) {
        vector.clear();
        self.ptr = vector.as_mut_ptr() as usize;
        self.length = vector.len();
        self.capacity = vector.capacity();
        mem::forget(vector);
    }

    /// May only be called once.
    /// Must be called with the same data-type as the `new` function was called with.
    unsafe fn drop<T>(&mut self) {
        self.wake_up::<T>();
    }
}

pub trait Transparent {
    type Inner;
    fn get(&self) -> &Self::Inner;
    fn get_mut(&mut self) -> &mut Self::Inner;
}
