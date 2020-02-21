use super::{AudioReader, AudioWriter, MidiWriter};
use crate::event::{DeltaEvent, RawMidiEvent};
use std::marker::PhantomData;

pub struct AudioDummy<S> {
    _phantom: PhantomData<S>,
}

impl<S> AudioDummy<S> {
    pub fn new() -> Self {
        AudioDummy {
            _phantom: PhantomData,
        }
    }
}

impl<S> AudioReader<S> for AudioDummy<S> {
    type Err = std::convert::Infallible;
    fn number_of_channels(&self) -> usize {
        0
    }

    fn frames_per_second(&self) -> u64 {
        44100
    }

    fn fill_buffer(&mut self, _output: &mut [&mut [S]]) -> Result<usize, Self::Err> {
        Ok(0) // TODO: Have a look at this implementation again: is this logical?
    }
}

impl<S> AudioWriter<S> for AudioDummy<S> {
    type Err = std::convert::Infallible;
    fn write_buffer(&mut self, _buffer: &[&[S]]) -> Result<(), Self::Err> {
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
