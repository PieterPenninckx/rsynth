use super::{AudioReader, AudioWriter, MidiReader, MidiWriter};
use crate::event::{DeltaEvent, RawMidiEvent};
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
    type Err = std::convert::Infallible;
    fn number_of_channels(&self) -> usize {
        0
    }

    fn frames_per_second(&self) -> u64 {
        44100
    }

    fn fill_buffer(&mut self, _output: &mut [&mut [F]]) -> Result<usize, Self::Err> {
        Ok(0) // TODO: Have a look at this implementation again: is this logical?
    }
}

impl<F> AudioWriter<F> for AudioDummy<F> {
    type Err = std::convert::Infallible;
    fn write_buffer(&mut self, _buffer: &[&[F]]) -> Result<(), Self::Err> {
        Ok(())
    }
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
