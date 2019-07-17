//! # Rsynth
//! The `rsynth` crate makes it easier to write software synthesizers.
//!
//! # The `Plugin` trait
//! The functionality of the plugin that is common to all back-ends is defined
//! by the [`Plugin`] trait. It defines some meta-data about the plugin and contains
//! the methods for rendering the audio.
//!
//! # The `EventHandler` trait
//! Plugins also implement `EventHandler` for each event type that they support.
//! Currently supported events are:
//!
//! * [`RawMidiEvent`]
//! * [`SysExEvent`]
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
//! [`Plugin`]: ./trait.Plugin.html
//! [`jack`]: ./backend/jack_backend/index.html
//! [`vst`]: ./backend/vst_backend/index.html
//! [`EventHandler`]: ./event/trait.EventHandler.html
//! [`RawMidiEvent`]: ./event/struct.RawMidiEvent.html
//! [`SysExEvent`]: ./event/struct.SysExEvent.html
//! [`render_buffer`]: ./trait.Plugin.html#tymethod.render_buffer
//! [`handle_event`]: ./event/trait.EventHandler.html#tymethod.handle_event

#[macro_use]
extern crate log;
extern crate asprim;
extern crate core;
extern crate num;
extern crate num_traits;

#[cfg(feature = "jack-backend")]
extern crate jack;
#[cfg(feature = "vst-backend")]
extern crate vst;

#[macro_use]
pub mod dev_utilities;
pub mod backend;
pub mod envelope;
pub mod event;
pub mod middleware;
pub mod note;

// Some notes about the design
// ===========================
//
// No `Default`
// ------------
// The `Default` trait is not required.
// Implementing `Default` is sometimes not possible with `#[derive(Default)]` and it feels
// awkward to implement setup (e.g. reading config files) in the `default()` method.
// For `rust-vst`, an extra macro wraps the setup in a `Default` implementation, so that at least it
// doesn't _feel_ awkward (but it's still a hack, of course).
// Also note that `rust-vst` only requires the `Default` trait to enable a default implementation
// for the `new()` function, it is not used directly by `rust-vst` itself.
//
// Not object safe
// ---------------
// Many of the traits are not object safe. In practice, this is not a problem for using `rust-vst`
// because an extra macro wraps it.
//
// Separate `EventHandler` trait
// -----------------------------
// There is a separate trait for event handling:
// ```
// trait EventHandler<E> {
//      fn handle_event(&mut self, event: E);
// }
// ```
// In this way, other crates can define their own event types.
//
//
// Associated constants for plugin meta-data
// -----------------------------------------
// The idea behind this is that it cannot change during the execution of the application.
// I'm not sure if this was really a good idea, e.g. `MAX_NUMBER_OF_AUDIO_INPUTS` may be
// read from a config file.
// We're leaving this as it is for now until we have a better understanding of the requirements
// for the meta-data (e.g. when we add support for LV2).
//
// Separate `AudioRenderer` and `ContextualAudioRenderer` traits
// -------------------------------------------------------------
// These methods were originally together with some meta-data in the `Plugin` trait,
// but we have split this off so that backends can have special meta-data, without
// interfering with the rendering.
//
// Generic trait instead of generic method
// ---------------------------------------
// The `AudioRenderer` and `ContextualAudioRenderer` traits are generic over the floating
// point type, instead of having a method that is generic over _all_ float types.
// In practice, backends only require renderers over f32 and/or f64, not over _all_ floating
// point types. So in practice, for instance the vst backend can require
// `AudioRenderer<f32>` and `AudioRenderer<f64>`. These can be implemented separately,
// allowing for SIMD optimization, or together in one generic impl block.
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

/// Set the maximum number of inputs and the maximum number of outputs.
/// Also defines how sample rate changes are handled.
// TODO: Find a better name for this trait.
pub trait AudioRendererMeta {
    /// The maximum number of inputs supported.
    const MAX_NUMBER_OF_AUDIO_INPUTS: usize;

    /// The maximum number of audio outputs.
    const MAX_NUMBER_OF_AUDIO_OUTPUTS: usize;

    /// Called when the sample-rate changes.
    /// The backend should ensure that this function is called before
    /// any other.
    ///
    /// # Parameters
    /// `sample_rate`: The new sample rate in frames per second (Hz).
    /// Common sample rates are 44100 Hz (CD quality)
    fn set_sample_rate(&mut self, sample_rate: f64);
}

/// This function is the core of the plugin.
/// It is called repeatedly for subsequent buffers.
/// The lengths of all elements of `inputs` and the lengths of all elements of `outputs`
/// are all guaranteed to equal to each other.
/// This shared length can however be different for subsequent calls to `render_buffer`.
pub trait AudioRenderer<F>: AudioRendererMeta {
    /// This method called repeatedly for subsequent buffers.
    /// You may assume that the number of inputs (`inputs.len()`) is smaller than or equal to `Self::MAX_NUMBER_OF_AUDIO_INPUTS`.
    /// You may assume that the number of outputs (`outputs.len()`) is smaller than or equal to `Self::MAX_NUMBER_OF_AUDIO_OUTPUTS`.
    fn render_buffer(&mut self, inputs: &[&[F]], outputs: &mut [&mut [F]]);
}

/// Similar to `AudioRenderer`, but with an additional `context` parameter.
pub trait ContextualAudioRenderer<F, Context>: AudioRendererMeta {
    fn render_buffer(&mut self, inputs: &[&[F]], outputs: &mut [&mut [F]], context: &mut Context);
}

/// The trait that all plugins need to implement.
pub trait Plugin: AudioRendererMeta {
    /// The name of the plugin.
    const NAME: &'static str;

    /// The name of the audio input with the given index.
    /// `index` must be strictly smaller than `Self::MAX_NUMBER_OF_AUDIO_INPUTS`
    ///
    /// # Note
    /// When using the Jack backend, you may not provide an empty string.
    fn audio_input_name(index: usize) -> String {
        format!("audio in {}", index)
    }

    /// The name of the audio output with the given index.
    /// `index` must be strictly smaller than `Self::MAX_NUMBER_OF_AUDIO_OUTPUTS`
    ///
    /// # Note
    /// When using the Jack backend, you may not provide an empty string.
    fn audio_output_name(index: usize) -> String {
        format!("audio out {}", index)
    }
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
