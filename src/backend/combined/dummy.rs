//! Dummy backend that does nothing, useful for testing.
use crate::event::{DeltaEvent, RawMidiEvent};
use core::cmp;
use std::marker::PhantomData;

/// Dummy backend that does nothing, useful for testing and e.g. for offline renderers
/// that have no audio input or output.
pub struct AudioDummy<S> {
    _phantom: PhantomData<S>,
    frames_per_second: u32,
    length_in_frames: usize,
    number_of_channels: usize,
}

impl<S> AudioDummy<S> {
    /// Create a new `AudioDummy` with the given sample rate, in frames per second.
    pub fn new(frames_per_second: u32, length_in_frames: usize, number_of_channels: usize) -> Self {
        Self {
            frames_per_second,
            length_in_frames,
            number_of_channels,
            _phantom: PhantomData,
        }
    }
}

pub struct MidiDummy {}

impl MidiDummy {
    pub fn new() -> Self {
        MidiDummy {}
    }
}
