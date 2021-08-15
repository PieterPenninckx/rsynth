//! Dummy backend that does nothing, useful for testing.
use super::{AudioReader, AudioWriter, MidiWriter};
use crate::buffer::{AudioBufferIn, AudioBufferOut};
use crate::event::{DeltaEvent, RawMidiEvent};
use core::cmp;
use std::marker::PhantomData;

/// Dummy backend that does nothing, useful for testing and e.g. for offline renderers
/// that have no audio input or output.
pub struct AudioDummy<S> {
    _phantom: PhantomData<S>,
    frames_per_second: u32,
    length_in_frames: usize,
}

impl<S> AudioDummy<S> {
    /// Create a new `AudioDummy` with the given sample rate, in frames per second.
    pub fn with_sample_rate_and_length(frames_per_second: u32, length_in_frames: usize) -> Self {
        Self {
            frames_per_second,
            length_in_frames,
            _phantom: PhantomData,
        }
    }
}

impl<S> AudioReader<S> for AudioDummy<S>
where
    S: Copy,
{
    type Err = std::convert::Infallible;
    fn number_of_channels(&self) -> usize {
        0
    }

    fn frames_per_second(&self) -> u64 {
        self.frames_per_second as u64
    }

    fn fill_buffer(&mut self, output: &mut AudioBufferOut<S>) -> Result<usize, Self::Err> {
        let number_of_frames_written = cmp::min(self.length_in_frames, output.number_of_frames());
        self.length_in_frames -= number_of_frames_written;
        Ok(number_of_frames_written)
    }
}

impl<S> AudioWriter<S> for AudioDummy<S>
where
    S: Copy,
{
    type Err = std::convert::Infallible;
    fn write_buffer(&mut self, _buffer: &AudioBufferIn<S>) -> Result<(), Self::Err> {
        Ok(())
    }
}

pub struct MidiDummy {}

impl MidiDummy {
    pub fn new() -> Self {
        MidiDummy {}
    }
}

impl Iterator for MidiDummy {
    type Item = DeltaEvent<RawMidiEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl MidiWriter for MidiDummy {
    fn write_event(&mut self, _event: DeltaEvent<RawMidiEvent>) {}
}
