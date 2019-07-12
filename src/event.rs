//! This module defines the `EventHandler` trait and some event types: `RawMidiEvent`,
//! `SysExEvent`, ...
#[cfg(feature = "stable")]
use crate::dev_utilities::compatibility::*;
#[cfg(feature = "stable")]
use syllogism::{Distinction, Specialize};
#[cfg(feature = "stable")]
use syllogism_macro::impl_specialization;

/// The trait that plugins should implement in order to handle the given type of events.
///
/// The type parameter `E` corresponds to the type of the event.
/// The type parameter `C` corresponds to the [context] of the plugin.
///
/// [context]: ../context/index.html
pub trait EventHandler<E, C> {
    fn handle_event(&mut self, event: E, context: &mut C);
}

/// A System Exclusive ("SysEx") event.
#[derive(Clone, Copy)]
pub struct SysExEvent<'a> {
    data: &'a [u8],
}

impl<'a> SysExEvent<'a> {
    /// Create a new `SysExEvent` with the given `data`.
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
}

/// A raw midi event.
#[derive(Clone, Copy)]
pub struct RawMidiEvent {
    data: [u8; 3],
}

impl RawMidiEvent {
    /// Create a new `RawMidiEvent` with the given raw data.
    pub fn new(data: [u8; 3]) -> Self {
        Self { data }
    }
    /// Get the raw data from a `RawMidiEvent`.
    pub fn data(&self) -> &[u8; 3] {
        &self.data
    }
}

/// `Timed<E>` adds timing to an event.
#[derive(PartialEq, Eq, Debug)]
pub struct Timed<E> {
    /// The offset (in frames) of the event.
    /// E.g. when `time_in_frames` is 6, this means that
    /// the event happens on the sixth frame of the buffer in the call to
    /// the [`render_buffer`] method of the `Plugin` trait.
    ///
    /// [`render_buffer`]: ../trait.Plugin.html#tymethod.render_buffer
    pub time_in_frames: u32,
    /// The underlying event.
    pub event: E,
}

impl<E> Timed<E> {
    pub fn new(time_in_frames: u32, event: E) -> Self {
        Self {
            time_in_frames,
            event,
        }
    }
}

impl<E> Clone for Timed<E>
where
    E: Clone,
{
    fn clone(&self) -> Self {
        Timed {
            time_in_frames: self.time_in_frames,
            event: self.event.clone(),
        }
    }
}

impl<E> Copy for Timed<E> where E: Copy {}

#[cfg(feature = "stable")]
impl<E, T> Specialize<Timed<T>> for Timed<E>
where
    E: Specialize<T>,
{
    fn specialize(self) -> Distinction<Timed<T>, Self> {
        let Timed {
            time_in_frames,
            event,
        } = self;
        match event.specialize() {
            Distinction::Generic(g) => Distinction::Generic(Timed {
                time_in_frames,
                event: g,
            }),
            Distinction::Special(s) => Distinction::Special(Timed {
                time_in_frames,
                event: s,
            }),
        }
    }
}

#[cfg(feature = "stable")]
impl_specialization!(
    trait NotInCrateRsynth;
    macro macro_for_rsynth;

    type SysExEvent<'a>;
    type RawMidiEvent;
    type Timed<E>;
);
