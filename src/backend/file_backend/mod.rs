use crate::dev_utilities::create_buffers;
use crate::event::{RawMidiEvent, Timed};
use crate::AudioRenderer;
use num_traits::Zero;
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
    // TODO: What if the writer gets an unexpected number of channels?
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

impl<F, AudioIn, AudioOut, MidiIn, MidiOut> FileBackend<F, AudioIn, AudioOut, MidiIn, MidiOut>
where
    AudioIn: AudioReader<F>,
    AudioOut: AudioReader<F>,
    MidiIn: MidiReader,
    MidiOut: MidiWriter,
    F: Zero,
{
    pub fn new(audio_in: AudioIn, audio_out: AudioOut, midi_in: MidiIn, midi_out: MidiOut) -> Self {
        Self {
            audio_in,
            audio_out,
            midi_in,
            midi_out,
            _phantom: PhantomData,
        }
    }

    pub fn run<R>(&mut self, mut plugin: R, buffer_size: usize)
    where
        R: AudioRenderer<F>,
    {
        assert!(buffer_size > 0);
        let number_of_channels = self.audio_in.number_of_channels();

        assert!(number_of_channels > 0);
        let input_buffers = create_buffers(number_of_channels, buffer_size);
        let mut output_buffers = create_buffers(number_of_channels, buffer_size);

        loop {
            let input: Vec<&[F]> = input_buffers.iter().map(|b| b.as_slice()).collect();
            let mut output: Vec<&mut [F]> = output_buffers
                .iter_mut()
                .map(|b| b.as_mut_slice())
                .collect();
            plugin.render_buffer(input.as_slice(), output.as_mut_slice());
        }
    }
}
