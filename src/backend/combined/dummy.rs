use super::{AudioReader, AudioWriter, DeltaEvent, MidiReader, MidiWriter};
use crate::event::RawMidiEvent;
use std::marker::PhantomData;

pub struct AudioDummy<F> {
    _phantom: PhantomData<F>,
}

impl<F> AudioDummy<F> {
    pub fn new() -> Self {
        AudioDummy {
            _phantom: PhantomData,
        }
    }
}

impl<F> AudioReader<F> for AudioDummy<F> {
    fn number_of_channels(&self) -> usize {
        0
    }

    fn frames_per_second(&self) -> u64 {
        44100
    }

    fn fill_buffer(&mut self, _output: &mut [&mut [F]]) -> usize {
        0
    }
}

impl<F> AudioWriter<F> for AudioDummy<F> {
    fn write_buffer(&mut self, _buffer: &[&[F]]) {}
}

pub struct MidiDummy {}

impl MidiDummy {
    pub fn new() -> Self {
        MidiDummy {}
    }
}

impl MidiReader for MidiDummy {
    fn read_event(&mut self) -> Option<DeltaEvent<RawMidiEvent>> {
        None
    }
}

impl MidiWriter for MidiDummy {
    fn write_event(&mut self, _event: DeltaEvent<RawMidiEvent>) {}
}
