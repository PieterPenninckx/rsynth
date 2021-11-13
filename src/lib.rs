#![feature(trace_macros)]
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
//! ## How it works
//! `rsynth` uses "ports" for audio channels, midi channels, "CV" (control voltage) etc.
//! You define a custom struct containing the ports you need, e.g.
//! ```
//! use rsynth::event::{Timed, RawMidiEvent};
//!
//! struct MyPorts<'a> {
//!     audio_out_left: &'a mut [f32],
//!     audio_out_right: &'a mut [f32],
//!     midi_in: &'a mut dyn Iterator<Item = Timed<RawMidiEvent>>
//! }
//! ```
//!
//! Each field of the struct corresponds to one port.
//! The port type is defined by the data type of the field.
//! In the example `audio_out_left` has data type `&mut [f32]`, so this corresponds to an audio out port.
//! See [the documentation of the `derive_ports!` macro](`derive_ports!`) for more information.
//!
//! With this, you can already define your plugin or application:
//! ```
//! # use rsynth::event::{Timed, RawMidiEvent};
//! use rsynth::ContextualAudioRenderer;
//!
//! struct MyPorts<'a> {
//!     // ...
//! #    audio_out_left: &'a mut [f32],
//! #    audio_out_right: &'a mut [f32],
//! #    midi_in: &'a mut dyn Iterator<Item = Timed<RawMidiEvent>>
//! }
//!
//! struct MyPlugin {
//!    // Fields go here.
//! }
//!
//! impl<'a, Context> ContextualAudioRenderer<MyPorts<'a>, Context> for MyPlugin
//! {
//!     fn render_buffer(&mut self, _ports: MyPorts<'a>, _context: &mut Context) {
//!       // Implementation goes here.
//!     }
//! }
//! ```
//!
//! The precise traits that the plugin needs to implement are mostly independent of the API that
//! you want to use. But there may be exceptions.
//! E.g. if you want to use `jack`, your plugin may need to implement some additional traits.
//!
//! The API's that `rsynth` abstracts over, don't (always) offer constructing this struct in this way.
//! The API's offer another way to represent the ports, and the custom struct you defined for containing
//! the ports (e.g. `MyPorts`) needs to be constructed from the API-specific data-representation.
//!
//! Constructing the custom struct containing the ports from the API-specific data representation
//! is the task for a so called "builder". You can define the "builder" yourself, but you don't have to.
//! You can use [`derive_ports!`] macro for that.
//!
//! Example:
//! ```
//! use rsynth::{derive_ports, event::{Timed, RawMidiEvent}};
//! #[cfg(feature = "backend-jack")]
//! use rsynth::derive_jack_port_builder;
//!
//! derive_ports! {
//!   struct MyPorts<'a> {
//!       audio_out_left: &'a mut [f32],
//!       audio_out_right: &'a mut [f32],
//!       midi_in: &'a mut dyn Iterator<Item = Timed<RawMidiEvent>>
//!   }
//!
//!    #[cfg(feature = "backend-jack")]
//!    derive_jack_port_builder! {
//!        struct MyPortsBuilder {
//!            generate_fields!()
//!        }
//!    }
//! }
//! ```
//!
//! The "builder" generates the custom struct for the ports from the API-specific data representation.
//!
//! The custom plugin or application implements traits that are mostly API-independent.
//! However, each API defines its own traits that a plugin or application needs to implement.
//! To connect the two, we use a so-called "delegator".
//! The delegator is generic over the data type of the builder and the data type of teh plugin.
//! The delegator has two fields: one field to contain the builder and one field to contain the plugin.
//! Under some conditions (see below), the delegator implements the API-specific traits. It does so by
//! 1) First using the builder to construct the custom data representation of the ports from the generic data representation of the ports.
//! 2) Then calling the appropriate method on the plugin, supplying the custom data representation.
//!
//! You don't need to implement the "delegator" yourself: it's defined by `rsynth`.
//! The precise "delegator" you can use depends on the API you want to abstract over.
//! E.g. for the jack API, you can use the [`JackHandler`] delegator.
//!
//! Roughly speaking, the delegator only implements the API-specific traits when the plugin
//! implements the generic traits.
//! The details are a little more complicated, but it boils down no the plugin implementing
//! the appropriate traits.
//! E.g. for the the generic struct `JackHandler<B, P>` only implements the jack-specific traits
//! when the type parameter `B` that corresponds to the builder implements
//! `DelegateHandling<P, (&'a Client, &'a ProcessScope), Output = Control>`.
//! The (macro-generated code for) the builder only implements the `DelegateHandling<P, _>` trait
//! when `P` implements the `ContextualAudioRenderer<MyPorts, Client>` trait.
//!
//! So to summarise:
//!
//! The API-independent part:
//! 1) You define a custom struct for holding the ports and put this impl block in the
//! [`derive_ports!`] macro.
//! 2) You define a custom plugin.
//! 3) You implement the correct, mostly API-independent traits for the custom plugin (e.g. the [`ContextualAudioRenderer`] trait).
//! 4) For each backend that you want to use, you instruct the [`derive_ports!`] to generate
//!    code for a builder.
//!
//! For each API, you write the following API-dependent code:
//! 1) Construct the plugin.
//! 2) Construct the (API-dependent) builder.
//! 3) Construct the (API-dependent) delegator from the plugin and the builder.
//! 4) Pass the delegator to the API.
//!
//! Together, the example looks as follows:
//! TODO

#[macro_use]
extern crate log;

use std::fmt::{Error, Write};

use crate::meta::{AudioPort, General, Meta, MidiPort, Name, Port};

#[macro_use]
pub mod buffer;
pub mod backend;
pub mod event;
pub mod meta;
pub mod test_utilities;

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

/// Render audio with the given ports and a given context.
/// Plugins and applications implement this trait.
/// The type parameter `Ports` can typically be constructed with the [`derive_ports!`] macro.
pub trait ContextualAudioRenderer<Ports, Context> {
    /// Render audio with the given ports and context.
    fn render_buffer(&mut self, ports: Ports, context: &mut Context);
}

/// Delegate the handling of some backend-specific data to a generic plugin or applications.
/// This trait is used to
pub trait DelegateHandling<P, D> {
    type Output;
    fn delegate_handling(&mut self, p: &mut P, d: D) -> Self::Output;
}
