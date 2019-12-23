//! # Rsynth
//! A crate for developing audio plugins and applications in Rust, with a focus on software synthesis.
//! Rsynth is well suited as a bootstrap for common audio plugin generators.
//! It handles voices, voice-stealing, polyphony, etc. so the programmer's main focus can be DSP.
//!
//! ## Back-ends
//! `rsynth` currently supports the following back-ends:
//!
//! * [`combined`] combine different back-ends for audio input, audio output, midi input and
////!     midi output, mostly for offline rendering and testing (behind various features)
//! * [`jack`] (behind the `backend-jack` feature)
//! * [`vst`] (behind the backend-vst)
//!
//! See the documentation of each back-end for more information.
//!
//! ## Rendering audio
//! Audio can be rendered with the [`ContextualAudioRenderer`] trait that is generic over the floating
//! point type (`f32` or `f64`). The parameter `context` is used by the
//! host or environment to pass extra data.
//!
//! The plugin or application can internally also use the [`AudioRenderer`] trait, which is similar
//! to the [`ContextualAudioRenderer`] trait, but does not have a `context` parameter.
//!
//! ## Meta-data
//! There are a number of traits to define some meta-data.
//! Every plugin should implement these, but it can be tedious, so you can implement these
//! traits in a more straightforward way by implementing the [`Meta`] trait.
//! However, you can also implement these trait "by hand":
//!
//! * [`CommonPluginMeta`]
//!     * Name of the plugin etc
//! * [`AudioHandlerMeta`]
//!     * Number of audio ports
//! * [`MidiHandlerMeta`]
//!     * Number of midi ports
//! * [`CommonAudioPortMeta`]
//!     * Names of the audio in and out ports
//! * [`CommonPluginMeta`]
//!     * Name of the plugin or application
//!
//!
//! ## Handling events
//! Plugins and applications can also implement [`ContextualEventHandler`] and [`EventHandler`]
//! for each event type that they support.
//! Currently supported events are:
//!
//! * [`RawMidiEvent`]
//! * [`SysExEvent`]
//! * [`Timed<T>`]
//! * [`Indexed<T>`]
//!
//! ## Utilities
//! Utilities are are types that you can include to perform several common tasks for the
//! plugin or application:
//!
//! * polyphony: managing of different voices
//! * timesplitting: split the audio buffer at the events
//!
//! ## Some audio concepts
//! A *sample* is a single number representing the air pressure at a given time.
//! It is usually represented by an `f32`, `f64`, `i16` or `i32` number, but other
//! types are possible as well.
//!
//! A *channel* usually corresponds with a speaker or a number of speakers.
//! E.g. in a stereo setup, there is a "left" channel and a "right" channel.
//!
//! A *frame* consists of the samples for all the channels at a given time.
//!
//! A *buffer* consists of subsequent samples for a given channel and corresponds
//! to a certain time period.
//! (Non-standard terminology.)
//!
//! A *chunk* consists of the buffers for all channels for a given time period.
//! (Non-standard terminology.)
//!
//!```text
//!                         ┌ chunk     ┌ frame
//!             ┌ sample    ↓           ↓
//!             │      ┌─────────┐     ┌─┐
//!          ┌──↓──────┼─────────┼─────┼─┼───────────────────┐
//! channel →│• • • • •│• • • • •│• • •│•│• • • • • • • • • •│
//!          └─────────┼─────────┼─────┼─┼───────────────────┘
//!           • • • • •│• • • • •│• • •│•│• • • • • • • • • •
//!                    │         │     │ │   ┌───────┐
//!           • • • • •│• • • • •│• • •│•│• •│• • • •│• • • •
//!                    └─────────┘     └─┘   └───────┘
//!                                            ↑
//!                                            └ buffer
//! ```
//!
//! [`Plugin`]: ./trait.Plugin.html
//! [`jack`]: ./backend/jack_backend/index.html
//! [`vst`]: ./backend/vst_backend/index.html
//! [`combined`]: ./backend/combined/index.html
//! [`EventHandler`]: ./event/trait.EventHandler.html
//! [`RawMidiEvent`]: ./event/struct.RawMidiEvent.html
//! [`SysExEvent`]: ./event/struct.SysExEvent.html
//! [`Timed<T>`]: ./event/struct.Timed.html
//! [`Indexed<T>`]: ./event/struct.Indexed.html
//! [`render_buffer`]: ./trait.Plugin.html#tymethod.render_buffer
//! [`handle_event`]: ./event/trait.EventHandler.html#tymethod.handle_event
//! [`CommonPluginMeta`]: ./trait.CommonPluginMeta.html
//! [`AudioHandlerMeta`]: ./trait.AudioHandlerMeta.html
//! [`MidiHandlerMeta`]: ./trait.MidiHandlerMeta.html
//! [`CommonAudioPortMeta`]: ./trait.CommonAudioPortMeta.html
//! [`Meta`]: ./metaconfig/trait.Meta.html
//! [`AudioRenderer`]: ./trait.AudioRenderer.html
//! [`ContextualEventHandler`]: ./event/trait.ContextualEventHandler.html
//! [`EventHandler`]: ./event/trait.EventHandler.html

#[macro_use]
extern crate log;
extern crate asprim;
extern crate num_traits;
extern crate vecstorage;

#[cfg(feature = "backend-file-hound")]
extern crate hound;
#[cfg(feature = "backend-jack")]
extern crate jack;
#[cfg(feature = "backend-file-hound")]
extern crate sample;
#[cfg(feature = "backend-vst")]
extern crate vst;

#[macro_use]
extern crate doc_comment;

use crate::metaconfig::{AudioPort, General, Meta, MidiPort, Name, Port};

#[macro_use]
pub mod dev_utilities;
pub mod backend;
pub mod envelope;
pub mod event;
pub mod middleware;
pub mod utilities;

doctest!("../README.md");

pub mod metaconfig;

// Notes about the design
// ======================
//
// The `Default` trait is not required
// -----------------------------------
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
// In this way, third party crates that define backends can define their own event types.
//
//
// No associated constants for plugin meta-data
// --------------------------------------------
// The idea behind this was that it cannot change during the execution of the application.
// We got rid of this in order to enable a more dynamic approach and in order to enable the
// `Meta` trait.
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
// This has been solved by the `VecStorage` struct, which has moved to its own crate.
//
// One remaining issue is that the length of the buffer cannot be known when there are 0 inputs and
// 0 outputs.
//
// Events
// ------
// Currently, backends that support one MIDI-port use the `Timed<RawMidiEvent>` type
// and backends that support moree MIDI-ports use the `Indexed<Timed<RawMidiEvent>>` type.

/// Define the maximum number of audioinputs and the maximum number of audio outputs.
/// This trait can be more conveniently implemented by implementing the [`Meta`] trait.
///
/// [`Meta`]: ./metaconfig/trait.Meta.html
pub trait AudioHandlerMeta {
    /// The maximum number of audio inputs supported.
    /// This method should return the same value every time it is called.
    fn max_number_of_audio_inputs(&self) -> usize;

    /// The maximum number of audio outputs supported.
    /// This method should return the same value every time it is called.
    fn max_number_of_audio_outputs(&self) -> usize;
}

/// Define how sample-rate changes are handled.
pub trait AudioHandler: AudioHandlerMeta {
    /// Called when the sample-rate changes.
    /// The backend should ensure that this function is called before
    /// any other.
    ///
    /// # Parameters
    /// `sample_rate`: The new sample rate in frames per second (Hz).
    /// Common sample rates are 44100 Hz (CD quality) and 48000 Hz,
    /// commonly used for video production.
    // TODO: Looking at the WikiPedia list https://en.wikipedia.org/wiki/Sample_rate, it seems that
    // TODO: there are no fractional sample rates. Maybe change the data type into u32?
    fn set_sample_rate(&mut self, sample_rate: f64);
}

/// Define the maximum number of midi inputs and the maximum number of midi outputs.
/// This trait can be more conveniently implemented by implementing the [`Meta`] trait.
///
/// [`Meta`]: ./metaconfig/trait.Meta.html
pub trait MidiHandlerMeta {
    /// The maximum number of midi inputs supported.
    /// This method should return the same value for subsequent calls.
    fn max_number_of_midi_inputs(&self) -> usize;
    /// The maximum number of midi outputs supported.
    /// This method should return the same value for subsequent calls.
    fn max_number_of_midi_outputs(&self) -> usize;
}

/// Defines how audio is rendered.
///
/// The type parameter `F` refers to the floating point type.
/// It is typically `f32` or `f64`.
pub trait AudioRenderer<F>: AudioHandler {
    /// This method is called repeatedly for subsequent buffers.
    ///
    /// You may assume that the number of inputs (`inputs.len()`)
    /// is smaller than or equal to [`Self::max_number_of_audio_inputs()`].
    /// You may assume that the number of outputs (`outputs.len()`)
    /// is smaller than or equal to [`Self::max_number_of_audio_outputs()`].
    ///
    /// The lengths of all elements of `inputs` and the lengths of all elements of `outputs`
    /// are all guaranteed to equal to each other.
    /// This shared length can however be different for subsequent calls to `render_buffer`.
    fn render_buffer(&mut self, inputs: &[&[F]], outputs: &mut [&mut [F]]);
}

/// Defines how audio is rendered, similar to the `AudioRenderer` trait.
/// The extra parameter `context` can be used by the backend to provide extra information.
///
/// See the documentation of [`AudioRenderer`] for more information.
pub trait ContextualAudioRenderer<F, Context>: AudioHandler {
    /// This method called repeatedly for subsequent buffers.
    ///
    /// It is similar to the [`render_buffer`] from the [`AudioRenderer`] trait,
    /// see its documentation for more information.
    fn render_buffer(&mut self, inputs: &[&[F]], outputs: &mut [&mut [F]], context: &mut Context);
}

/// Provides common meta-data of the plugin or application to the host.
/// This trait is common for all backends that need this info.
/// This trait can be more conveniently implemented by implementing the [`Meta`] trait.
///
/// [`Meta`]: ./metaconfig/trait.Meta.html
pub trait CommonPluginMeta {
    /// The name of the plugin or application.
    fn name<'a>(&'a self) -> &'a str;
}

/// Provides some meta-data of the audio-ports used by the plugin or application to the host.
/// This trait can be more conveniently implemented by implementing the [`Meta`] trait.
///
/// [`Meta`]: ./metaconfig/trait.Meta.html
pub trait CommonAudioPortMeta: AudioHandlerMeta {
    /// The name of the audio input with the given index.
    /// You can assume that `index` is strictly smaller than [`Self::max_number_of_audio_inputs()`].
    ///
    /// # Note
    /// When using the Jack backend, this function should not return an empty string.
    ///
    /// [`Self::max_number_of_audio_inputs()`]: trait.AudioHandlerMeta.html#tymethod.max_number_of_audio_inputs
    fn audio_input_name(&self, index: usize) -> String {
        format!("audio in {}", index)
    }

    /// The name of the audio output with the given index.
    /// You can assume that `index` is strictly smaller than [`Self::max_number_of_audio_outputs()`].
    ///
    /// # Note
    /// When using the Jack backend, this function should not return an empty string.
    ///
    /// [`Self::max_number_of_audio_outputs()`]: ./trait.AudioHandlerMeta.html#tymethod.max_number_of_audio_outputs
    fn audio_output_name(&self, index: usize) -> String {
        format!("audio out {}", index)
    }
}

/// Provides some meta-data of the midi-ports used by the plugin or application to the host.
/// This trait can be more conveniently implemented by implementing the [`Meta`] trait.
///
/// [`Meta`]: ./metaconfig/trait.Meta.html
pub trait CommonMidiPortMeta: MidiHandlerMeta {
    /// The name of the midi input with the given index.
    /// You can assume that `index` is strictly smaller than [`Self::max_number_of_midi_inputs()`].
    ///
    /// # Note
    /// When using the Jack backend, this function should not return an empty string.
    ///
    /// [`Self::max_number_of_midi_inputs()`]: trait.MidiHandlerMeta.html#tymethod.max_number_of_midi_inputs
    fn midi_input_name(&self, index: usize) -> String {
        format!("midi in {}", index)
    }

    /// The name of the midi output with the given index.
    /// You can assume that `index` is strictly smaller than [`Self::max_number_of_midi_outputs()`]
    ///
    /// # Note
    /// When using the Jack backend, this function should not return an empty string.
    ///
    /// [`Self::max_number_of_midi_outputs()`]: ./trait.MidiHandlerMeta.html#tymethod.max_number_of_midi_outputs
    fn midi_output_name(&self, index: usize) -> String {
        format!("midi out {}", index)
    }
}

impl<T> CommonPluginMeta for T
where
    T: Meta,
    T::MetaData: General,
    <<T as Meta>::MetaData as General>::GeneralData: Name,
{
    fn name<'a>(&'a self) -> &'a str {
        self.meta().general().name()
    }
}

impl<T> AudioHandlerMeta for T
where
    T: Meta,
    T::MetaData: Port<AudioPort>,
{
    fn max_number_of_audio_inputs(&self) -> usize {
        self.meta().in_ports().len()
    }

    fn max_number_of_audio_outputs(&self) -> usize {
        self.meta().out_ports().len()
    }
}

impl<T> CommonAudioPortMeta for T
where
    T: Meta,
    T::MetaData: Port<AudioPort>,
    <<T as Meta>::MetaData as Port<AudioPort>>::PortData: Name,
{
    fn audio_input_name(&self, index: usize) -> String {
        self.meta().in_ports()[index].name().to_string()
    }

    fn audio_output_name(&self, index: usize) -> String {
        self.meta().out_ports()[index].name().to_string()
    }
}

impl<T> MidiHandlerMeta for T
where
    T: Meta,
    T::MetaData: Port<MidiPort>,
{
    fn max_number_of_midi_inputs(&self) -> usize {
        self.meta().in_ports().len()
    }

    fn max_number_of_midi_outputs(&self) -> usize {
        self.meta().out_ports().len()
    }
}

impl<T> CommonMidiPortMeta for T
where
    T: Meta,
    T::MetaData: Port<MidiPort>,
    <<T as Meta>::MetaData as Port<MidiPort>>::PortData: Name,
{
    fn midi_input_name(&self, index: usize) -> String {
        // TODO: It doesn't feel right that we have to do a `to_string` here.
        self.meta().in_ports()[index].name().to_string()
    }

    fn midi_output_name(&self, index: usize) -> String {
        // TODO: It doesn't feel right that we have to do a `to_string` here.
        self.meta().out_ports()[index].name().to_string()
    }
}
