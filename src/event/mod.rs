//! This module defines the `EventHandler` trait and some event types: `RawMidiEvent`,
//! `SysExEvent`, ...
use std::convert::{AsMut, AsRef};
use std::fmt::{Debug, Error, Formatter};

pub mod event_queue;

/// The trait that plugins should implement in order to handle the given type of events.
///
/// The type parameter `E` corresponds to the type of the event.
pub trait EventHandler<E> {
    fn handle_event(&mut self, event: E);
}

pub trait ContextualEventHandler<E, Context> {
    fn handle_event(&mut self, event: E, context: &mut Context);
}

/// A System Exclusive ("SysEx") event.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SysExEvent<'a> {
    data: &'a [u8],
}

impl<'a> Debug for SysExEvent<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "SysExEvent{{data (length: {:?}): &[", self.data.len())?;
        for byte in self.data {
            write!(f, "{:X} ", byte)?;
        }
        write!(f, "]}}")
    }
}

impl<'a> SysExEvent<'a> {
    /// Create a new `SysExEvent` with the given `data`.
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
    /// Get the data from the `SysExEvent`
    pub fn data(&self) -> &'a [u8] {
        self.data
    }
}

/// A raw midi event.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RawMidiEvent {
    data: [u8; 3],
}

impl Debug for RawMidiEvent {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "RawMidiEvent({:X} {:X} {:X})",
            self.data[0], self.data[1], self.data[2]
        )
    }
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

impl AsRef<Self> for RawMidiEvent {
    fn as_ref(&self) -> &RawMidiEvent {
        self
    }
}

impl AsMut<Self> for RawMidiEvent {
    fn as_mut(&mut self) -> &mut RawMidiEvent {
        self
    }
}

pub mod raw_midi_event_event_types {
    pub const RAW_MIDI_EVENT_EVENT_TYPE_MASK: u8 = 0xF0;
    pub const RAW_MIDI_EVENT_NOTE_OFF: u8 = 0x80;
    pub const RAW_MIDI_EVENT_NOTE_ON: u8 = 0x90;
    pub const RAW_MIDI_EVENT_NOTE_AFTERTOUCH: u8 = 0xA0;
    pub const RAW_MIDI_EVENT_CONTROLLER: u8 = 0xB0;
    pub const RAW_MIDI_EVENT_PROGRAM_CHANGE: u8 = 0xC0;
    pub const RAW_MIDI_EVENT_CHANNEL_AFTERTOUCH: u8 = 0xD0;
    pub const RAW_MIDI_EVENT_PITCH_BEND: u8 = 0xE0;
    pub const RAW_MIDI_EVENT_META: u8 = 0xF0;
}

/// `Timed<E>` adds timing to an event.
#[derive(PartialEq, Eq, Debug)]
pub struct Timed<E> {
    /// The offset (in frames) of the event relative to the start of
    /// the audio buffer.
    ///
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

impl<E> AsRef<E> for Timed<E> {
    fn as_ref(&self) -> &E {
        &self.event
    }
}

impl<E> AsMut<E> for Timed<E> {
    fn as_mut(&mut self) -> &mut E {
        &mut self.event
    }
}

/// `Indexed<E>` adds an index to an event.
#[derive(PartialEq, Eq, Debug)]
pub struct Indexed<E> {
    /// The index of the event
    pub index: usize,
    /// The underlying event.
    pub event: E,
}

impl<E> Indexed<E> {
    pub fn new(index: usize, event: E) -> Self {
        Self { index, event }
    }
}

impl<E> Clone for Indexed<E>
where
    E: Clone,
{
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            event: self.event.clone(),
        }
    }
}

impl<E> Copy for Indexed<E> where E: Copy {}

impl<E> AsRef<E> for Indexed<E> {
    fn as_ref(&self) -> &E {
        &self.event
    }
}

impl<E> AsMut<E> for Indexed<E> {
    fn as_mut(&mut self) -> &mut E {
        &mut self.event
    }
}
