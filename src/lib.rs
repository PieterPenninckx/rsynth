//! # Rsynth
//! An API abstraction for API's for audio plugins and applications.
//! Use it to write real-time audio effects, software synthesizers, ... and target different platforms
//! (vst, jack, offline audio rendering, ...).
//! It is currently most suitable for real-time or "streaming" audio processing.
//! E.g. you cannot use it to reverse audio in time.
//!
//! ## Back-ends
//! `rsynth` currently supports the following back-ends:
//!
//! * [`jack`] (behind the `backend-jack` feature)
//! * [`vst`] (behind the `backend-vst` feature)
//! * [`combined`] combine different back-ends for audio input, audio output, midi input and
//!     midi output, mostly for offline rendering and testing (behind various features)
//!
//! See the documentation of each back-end for more information.
//!
//! ## Features and how to use them
//!
//! `rsynth` puts common functionality of the different backends behind common traits.
//! Conversely, a plugin can be used for different backends by implementing common traits.
//! A mix-and-match approach is used: if a backend doesn't require a certain functionality,
//! you don't need the corresponding trait.
//!
//! ### Using the backend
//!
//! * Jack: [`run()`](./backend/jack_backend/fn.run.html)
//! * Offline : [`run()`](backend/combined/fn.run.html)
//! * VST 2.4:  [`vst_init!`]
//!
//! ## Rendering audio
//! All backends require the plugin/application to implement the [`ContextualAudioRenderer`] trait.
//! [`ContextualAudioRenderer`] has two type parameters and the type parameter depends on the
//! backends to use.
//! One type parameter is the data type used to represent a sample.
//! The other type parameter is called the "context" and can be used to access functionality of
//! the backend in the audio rendering itself.
//! Common functionality of the context is defined in the [`HostInterface`] trait.
//! The application or plugin can have either a generic implementation of the [`ContextualAudioRenderer`]
//! or choose to use different, specialized implementations if different behaviour is needed.
//!
//! ### Jack
//!
//! Applications need to implement
//! * [`AudioHandler`]
//! * [`ContextualAudioRenderer`]`<f32,`[`JackHost`]`>`
//!
//! ### Offline rendering
//! Applications need to implement
//! * [`ContextualAudioRenderer`]`<S, `[`MidiWriterWrapper`]`<`[`Timed`]`<`[`RawMidiEvent`]`>>>` Note: the type parameter `S`, which represents the sample data type, is free.
//!
//! ### VST 2.4
//! Plugins need to implement
//! * [`AudioHandler`]
//! * [`ContextualAudioRenderer`]`<f32,`[`HostCallback`]`>`
//! * [`ContextualAudioRenderer`]`<f64,`[`HostCallback`]`>`
//!
//! _Note_: [`HostCallback`] is re-exported from the VST crate, but implements `rsynth`'s
//! [`HostInterface`], which defines functionality shared by all backends.
//!
//!
//! ## Meta-data
//! There are a number of traits that an application or plugin needs to implement in order to define meta-data.
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
//! Additionally, back-ends can require extra trait related to meta-data.
//!
//! ## Handling events
//! Plugins or application can handle events by implementing a number of traits:
//!
//! * [`EventHandler`]
//! * [`ContextualEventHandler`]
//!
//! Both traits are generic over the event type.
//! These traits are very similar, the [`ContextualEventHandler`] trait adds one extra parameter
//! that defines a "context" that can be passed to the implementor of the trait, so that the
//! implementor of the trait does not need to own all data that is needed for handling the event;
//! it can also borrow some data with additional the `context` parameter.
//!
//! ### Events
//! `rsynth` defines a number of event types:
//!
//! * [`RawMidiEvent`]: a raw MIDI event
//! * [`SysExEvent`]: a system exclusive event
//! * [`Timed<T>`]: a generic event that associates a timestamp with the event
//! * [`Indexed<T>`]: a generic event that associates an index with the event
//!
//! [`jack`]: ./backend/jack_backend/index.html
//! [`vst`]: ./backend/vst_backend/index.html
//! [`combined`]: ./backend/combined/index.html
//! [`EventHandler`]: ./event/trait.EventHandler.html
//! [`RawMidiEvent`]: ./event/struct.RawMidiEvent.html
//! [`SysExEvent`]: ./event/struct.SysExEvent.html
//! [`Timed<T>`]: ./event/struct.Timed.html
//! [`Timed`]: ./event/struct.Timed.html
//! [`Indexed<T>`]: ./event/struct.Indexed.html
//! [`CommonPluginMeta`]: ./trait.CommonPluginMeta.html
//! [`AudioHandlerMeta`]: ./trait.AudioHandlerMeta.html
//! [`MidiHandlerMeta`]: ./trait.MidiHandlerMeta.html
//! [`CommonAudioPortMeta`]: ./trait.CommonAudioPortMeta.html
//! [`Meta`]: ./meta/trait.Meta.html
//! [`AudioRenderer`]: ./trait.AudioRenderer.html
//! [`ContextualAudioRenderer`]: trait.ContextualAudioRenderer.html
//! [`ContextualEventHandler`]: ./event/trait.ContextualEventHandler.html
//! [`EventHandler`]: ./event/trait.EventHandler.html
//! [`vst_init!`]: ./macro.vst_init.html
//! [`jack_backend::run()`]:  ./backend/jack_backend/fn.run.html
//! [`combined::run()`]: backend/combined/fn.run.html
//! [`HostCallback`]: ./backend/vst_backend/vst/plugin/struct.HostCallback.html
//! [`HostInterface`]: ./backend/trait.HostInterface.html
//! [`JackHost`]: ./backend/jack_backend/struct.JackHost.html
//! [`AudioHandler`]: ./trait.AudioHandler.html
//! [`MidiWriterWrapper`]: ./backend/combined/struct.MidiWriterWrapper.html

#[macro_use]
extern crate log;

use crate::buffer::AudioBufferInOut;
use crate::meta::{AudioPort, General, Meta, MidiPort, Name, Port};

#[macro_use]
pub mod buffer;
pub mod backend;
pub mod envelope;
pub mod event;
pub mod meta;
pub mod test_utilities;
pub mod utilities;

/// Re-exports from the `vecstorage` crate.
pub mod vecstorage {
    pub use vecstorage::VecStorage;
}

/// Define the maximum number of audio inputs and the maximum number of audio outputs.
///
/// Backends that require the plugin to implement this trait ensure that when calling the
/// [`render_buffer`] method of the [`AudioRenderer`] trait
/// *  the number of inputs channels (`buffer.number_of_input_channels()`) is smaller than or equal to
///    `Self::max_number_of_audio_inputs()` and
/// * the number of outputs (`buffer.number_of_output_channels()`) is smaller than or equal to
///    `Self::max_number_of_audio_outputs()`.
///
/// # Remark
/// This trait can be more conveniently implemented by implementing the [`Meta`] trait.
///
/// [`Meta`]: ./meta/trait.Meta.html
/// [`render_buffer`]: ./trait.AudioRenderer.html#tymethod.render_buffer
/// [`AudioRenderer`]: ./trait.AudioRenderer.html
pub trait AudioHandlerMeta {
    /// The maximum number of audio inputs supported.
    /// This method should return the same value every time it is called.
    fn max_number_of_audio_inputs(&self) -> usize;

    /// The maximum number of audio outputs supported.
    /// This method should return the same value every time it is called.
    fn max_number_of_audio_outputs(&self) -> usize;
}

/// Define how sample-rate changes are handled.
pub trait AudioHandler {
    /// Called when the sample-rate changes.
    /// The backend should ensure that this function is called before
    /// any other method.
    ///
    /// # Parameters
    /// `sample_rate`: The new sample rate in frames per second (Hz).
    /// Common sample rates are 44100 Hz (CD quality) and 48000 Hz.
    // TODO: Looking at the WikiPedia list https://en.wikipedia.org/wiki/Sample_rate, it seems that
    // TODO: there are no fractional sample rates. Maybe change the data type into u32?
    fn set_sample_rate(&mut self, sample_rate: f64);
}

/// Define the maximum number of midi inputs and the maximum number of midi outputs.
/// This trait can be more conveniently implemented by implementing the [`Meta`] trait.
///
/// [`Meta`]: ./meta/trait.Meta.html
pub trait MidiHandlerMeta {
    /// The maximum number of midi inputs supported.
    /// This method should return the same value for subsequent calls.
    fn max_number_of_midi_inputs(&self) -> usize;

    /// The maximum number of midi outputs supported.
    /// This method should return the same value for subsequent calls.
    fn max_number_of_midi_outputs(&self) -> usize;
}

// TODO: Is this trait actually used?
/// Defines how audio is rendered.
///
/// The type parameter `S` refers to the data type of a sample.
/// It is typically `f32` or `f64`.
pub trait AudioRenderer<S>
where
    S: Copy,
{
    /// This method is called repeatedly for subsequent audio buffers.
    fn render_buffer(&mut self, buffer: &mut AudioBufferInOut<S>);
}

/// Defines how audio is rendered, similar to the [`AudioRenderer`] trait.
/// The extra parameter `context` can be used by the backend to provide extra information.
///
/// The type parameter `S` refers to the data type of a sample.
/// It is typically `f32` or `f64`.
///
/// [`AudioRenderer`]: ./trait.AudioRenderer.html
pub trait ContextualAudioRenderer<S, Context>
where
    S: Copy,
{
    /// This method called repeatedly for subsequent buffers.
    ///
    /// It is similar to the [`render_buffer`] from the [`AudioRenderer`] trait,
    /// see its documentation for more information.
    ///
    /// [`AudioRenderer`]: ./trait.AudioRenderer.html
    /// [`render_buffer`]: ./trait.AudioRenderer.html#tymethod.render_buffer
    fn render_buffer(&mut self, buffer: &mut AudioBufferInOut<S>, context: &mut Context);
}

/// Provides common meta-data of the plugin or application to the host.
/// This trait is common for all backends that need this info.
/// This trait can be more conveniently implemented by implementing the [`Meta`] trait.
///
/// [`Meta`]: ./meta/trait.Meta.html
pub trait CommonPluginMeta {
    /// The name of the plugin or application.
    fn name(&self) -> &str;
}

/// Provides some meta-data of the audio-ports used by the plugin or application to the host.
/// This trait can be more conveniently implemented by implementing the [`Meta`] trait.
///
/// [`Meta`]: ./meta/trait.Meta.html
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
/// [`Meta`]: ./meta/trait.Meta.html
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
    fn name(&self) -> &str {
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
