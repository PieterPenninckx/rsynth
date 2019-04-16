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
//! [`Plugin`]: ./backend/trait.Plugin.html
//! [`jack`]: ./backend/jack_backend/index.html
//! [`vst`]: ./backend/vst_backend/index.html
//! [`Polyphony`]: ./middleware/polyphony/index.html
//! [`ZeroInit`]: ./middleware/zero_init/index.html
#[macro_use]
extern crate log;
extern crate asprim;
extern crate core;
#[cfg(feature = "jack-backend")]
extern crate jack;
extern crate num;
extern crate num_traits;
extern crate vst;

pub mod dev_utilities;
pub mod event;
pub mod backend;
pub mod dsp;
pub mod envelope;
pub mod middleware;
pub mod note;
pub mod point;
pub mod synth;

use asprim::AsPrim;
use num_traits::Float;

// Some notes about the design
// ===========================
// 
// No `Default`
// ------------
// `Plugin` doesn't need to implement the `Default` trait.
// Implementing `Default` is sometimes not possible with `#[derive(Default)]` and it feels
// awkward to implement setup (e.g. reading config files) in the `default()` method.
// For `rust-vst`, an extra macro wraps the setup in a `Default` implementation, so that at least it
// doesn't _feel_ awkward (but it's still a hack, of course).
// Also note that `rust-vst` only requires the `Default` trait to enable a default implementation
// for the `new()` function, it is not used directly by `rust-vst` itself.
//
// Not object safe
// ---------------
// The `Plugin` trait is not object safe. In practice, this is not a problem for using `rust-vst`
// because an extra macro wraps it.
//
// Type parameter for the event type
// ---------------------------------
// The current design has a type parameter for the event type in the `Plugin` trait.
// We are currently working on splitting `handle_event` to a separate trait
// ```
// trait EventHandler<E> {
//      fn handle_event(&mut self, event: E);
// }
// ```
// In this way, other crates can define their own event types.
// The block on that road is to allow the middleware to specialise for special event types.
// Rust doesn't support specialization (yet), so we're working on a work-around.
// 
// Associated constants for plugin meta-data
// -----------------------------------------
// The idea behind this is that it cannot change during the execution of the application.
// I'm not sure if this was really a good idea, e.g. `MAX_NUMBER_OF_AUDIO_INPUTS` may be 
// read from a config file. 
// We're leaving this as it is for now until we have a better understanding of the requirements
// for the meta-data (e.g. when we add support for LV2).
//
// Separate method for `set_sample_rate`
// -------------------------------------
// This is a separate method and not an "event type". The idea behind this is that it's guaranteed
// to be called before other methods and outside of the "realtime" path (whereas 
// `handle_events` is called in the "realtime" path).
// I don't know if this is the best solution, though. Leaving as it is until we have a more clear
// understanding of it.
// 
// Decisions behind `render_buffer`
// -------------------------------
// `render_buffer` is at the core and some design decisions made it the way it is now.
// 
// ### Generic over floating-point type
// This was in doomy's original design and we have kept it because it's handy in practice.
// It's probably problematic to allow for SIMD, so this will probably be changed when we have
// a more clear understanding of how portable SIMD is going to look like in stable Rust.
//
// ### Push-based (instead of pull-based)
// The `render_buffer` gets the buffers it needs as parameters instead of getting a queue from which
// it has to "pull" the input buffers (like Jack works and if I'm not mistaken AudioUnits as well).
// The upside is that it's straightforward from a developer perspective, the downside is that it's
// less flexible. E.g. it's hard to implement real-time sample rate conversion in this way.
// Nevertheless, I've chosen this design because it's what is convenient for most plugin developers
// and developers wanting to write something like real-time sample rate conversion will probably
// not use high-level abstractions like rsynth.
//
// ### Buffers as slices of slices
// Somewhere an intermediate design was to have traits `InputBuffer<'a>` and `OutputBuffer<'a>`,
// but this lead to a cascade of fights with the borrow checker:
//     * First it was problematic for the `Polyphonic` middleware (simplified pseudo-Rust of 
//      `Polyphonic`s `render_buffer` method):
//      ```
//      fn render_buffer<'a, I: InputBuffers<'a>, O: OutputBuffers<'a>>(&mut self, inputs: &I, outputs: &mut O) {
//           for voice in self.voices {
//               voice.render_buffer(inputs, outputs); // <-- the borrow of outputs needs to be shorter
//           }
//      }
//      ```
//      The compiler didn't allow this because the borrow of `outputs` must be shorter than the 
//      "external" lifetime `'a` in order to avoid overlapping borrows.
//
//    * Then we implemented it as follows:
//      ```
//      fn render_buffer<I, O>(&mut self, inputs: &I, outputs: &mut O) 
//      where for<'a> I: InputBuffers<'a>, O: OutputBuffers<'a>
//      {
//          // ...
//      }
//      ```
//      That solved one problem, but introduced `for<'a>` which is not a frequently used feature
//      in Rust and which is not supported in some contexts, so I ran into some trouble with this
//      (I've forgotten which).
// 
// For these reasons, I have abandoned this design and started using the slices instead.
// This in turn gives a problem for the API-wrappers, which will want to pre-allocate the buffer
// for the slices, but want to use this buffer for slices with different lifetimes.
// This has been solved by the `VecStorage` and `VecStorageMut` structs.
// 
// One remaining issue is that the length of the buffer cannot be known when there are 0 inputs and
// 0 outputs.
//
// Events
// ------
// Currently, only one MIDI-port is supported. This should be changed (e.g. Jack supports more
// than one MIDI-port). 
//
// Context
// -------
// Sometimes, plugins will want to e.g. share some data (e.g. samples) by all voices.
// In order to allow this, we're planning to add a `context` parameter (of generic type) to the
// `handle_events` and the `render_buffer` methods.
// We will typically want to use this context for many things and there are different parties
// involved:
// * The plugin and parameter-smoothing middleware may want to access the envelopes in the context
// * The voices may want to access shared data (e.g. samples)
// * the back-end and the plugin may want to use the context for some communication
// Because these involved parties potentially are defined in different crates, we want to have a
// mechanism to "add fields to a struct defined elsewhere".
// This is not possible of course, but we can define traits like `WithContext`, `WithSamples`,
// `WithHost` etc, compose structs and use blanket impls to make it all work. This sounds very
// handwavy, but we're already making good progress and the bulk of the complexity is for
// the back-end and the middleware: the plugin can simply 
// `impl Plugin<C> for MyPlugin where C: WithContext + WithSamples`
// etc., depending on what properties of the context the plugin wants to use.

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
