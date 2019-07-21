use super::{AudioReader, AudioWriter, MidiReader, MidiWriter};
use crate::event::{RawMidiEvent, Timed};
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

    fn fill_buffer(&mut self, output: &mut [&mut [F]]) -> usize {
        0
    }
}

impl<F> AudioWriter<F> for Dummy<F> {
    fn write_buffer(&mut self, buffer: &[&[F]]) {}
}

impl<F> MidiReader for Dummy<F> {
    fn read_event(&mut self) -> Option<Timed<RawMidiEvent>> {
        None
    }
}
