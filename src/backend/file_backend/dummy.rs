use super::{AudioReader, AudioWriter, MidiReader, MidiWriter};
use crate::backend::file_backend::DeltaEvent;
use crate::event::RawMidiEvent;
use std::marker::PhantomData;

pub struct Dummy<F> {
    _phantom: PhantomData<F>,
}

impl<F> Dummy<F> {
    pub fn new() -> Self {
        Dummy {
            _phantom: PhantomData,
        }
    }
}

impl<F> AudioReader<F> for Dummy<F> {
    fn number_of_channels(&self) -> usize {
        0
    }

    fn frames_per_second(&self) -> u32 {
        44100
    }

    fn fill_buffer(&mut self, output: &mut [&mut [F]]) -> usize {
        0
    }
}

impl<F> AudioWriter<F> for Dummy<F> {
    fn write_buffer(&mut self, buffer: &[&[F]]) {}
}

impl<F> MidiReader for Dummy<F> {
    fn read_event(&mut self) -> Option<DeltaEvent<RawMidiEvent>> {
        None
    }
}
