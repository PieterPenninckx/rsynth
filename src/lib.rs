//! # Rsynth
//! The `rsynth` crate makes it easier to write software synthesizers.
//!
//! # The `Plugin` trait
//! The functionality of the plugin that is common to all back-ends is defined
//! by the [`Plugin`] trait.
//!
//! # Back-ends
//! `rsynth` currently supports two back-ends:
//!
//! * [`jack`]
//! * [`vst`]
//!
//! In order to support a specific back-end, plugins may additionally need
//! to implement a backend-specific trait on top of the `Plugin` trait. See the
//! documentation of each back-end for more information.
//!
//! # Middleware
//! You can add features, such as polyphony, to your plug-in by using middleware.
//! Typically, suppose `M` is middleware and your plugin `P` implement the `Plugin` trait and
//! any other backend-specific trait, then `M<P>` also implements the `Plugin` trait
//! and the backend-specific traits `P` implements.
//! Currently, supported middleware is
//!
//! * [`Polyphony`]
//! * [`ZeroInit`]
//!
//! [`Plugin`]: ./trait.Plugin.html
//! [`jack`]: ./backend/jack_backend/index.html
//! [`vst`]: ./backend/vst_backend/index.html
//! [`Polyphony`]: ./middleware/polyphony/index.html
//! [`ZeroInit`]: ./middleware/zero_init/index.html

#![cfg_attr(not(feature = "stable"), feature(specialization, overlapping_marker_traits))]

#[macro_use]
extern crate log;
extern crate asprim;
extern crate core;
#[cfg(feature = "jack-backend")]
extern crate jack;
extern crate num;
extern crate num_traits;
extern crate vst;

#[cfg(feature = "stable")]
#[macro_use]
extern crate syllogism;
#[cfg(feature = "stable")]
extern crate syllogism_macro;

#[macro_use]
pub mod dev_utilities;
pub mod event;
pub mod context;
pub mod backend;
pub mod middleware;
pub mod note;

use asprim::AsPrim;
use num_traits::Float;

/// The trait that all plugins need to implement.
pub trait Plugin<C> {
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
    fn render_buffer<F>(&mut self, inputs: &[&[F]], outputs: &mut [&mut [F]], context: &mut C)
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
