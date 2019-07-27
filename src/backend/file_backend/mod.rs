use crate::event::{RawMidiEvent, Timed};
use std::marker::PhantomData;

pub mod dummy;
#[cfg(feature = "backend-file-hound")]
pub mod hound;

pub trait AudioReader<F> {
    fn number_of_channels(&self) -> usize;

    /// Fill the buffers. Return the number of frames that have been written.
    /// If it is `<` the number of frames in the input, now more frames can be expected.
    fn fill_buffer(&mut self, output: &mut [&mut [F]]) -> usize;
}

pub trait AudioWriter<F> {
    // TODO: This does not foresee error handling in any way ...
    fn write_buffer(&mut self, buffer: &[&[F]]);
}

pub trait MidiReader {
    /// Time is delta relative to the previous event.
    fn read_event(&mut self) -> Option<Timed<RawMidiEvent>>;
}

pub trait MidiWriter {
    /// Time is delta relative to the previous event.
    fn write_event(&mut self, event: Timed<RawMidiEvent>);
}

pub struct FileBackend<F, AudioIn, AudioOut, MidiIn, MidiOut>
where
    AudioIn: AudioReader<F>,
    AudioOut: AudioReader<F>,
    MidiIn: MidiReader,
    MidiOut: MidiWriter,
{
    audio_in: AudioIn,
    audio_out: AudioOut,
    midi_in: MidiIn,
    midi_out: MidiOut,
    _phantom: PhantomData<F>,
}
