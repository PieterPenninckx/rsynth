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
