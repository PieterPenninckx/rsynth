//! Defines the JACK backend and the VST backend.
use asprim::AsPrim;
use num_traits::Float;

#[cfg(feature = "jack-backend")]
pub mod jack_backend;
pub mod vst_backend;

pub mod event;

/// The trait that all plugins need to implement.
pub trait Plugin {
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
