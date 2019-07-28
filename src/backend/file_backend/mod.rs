use crate::dev_utilities::create_buffers;
use crate::event::event_queue::EventQueue;
use crate::event::{EventHandler, RawMidiEvent, Timed};
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

pub fn run<F, AudioIn, AudioOut, MidiIn, MidiOut, R>(
    mut plugin: R,
    buffer_size: usize,
    audio_in: AudioIn,
    audio_out: AudioOut,
    event_queue_capacity: usize,
    mut midi_in: MidiIn,
    mut midi_out: MidiOut,
) where
    AudioIn: AudioReader<F>,
    AudioOut: AudioReader<F>,
    MidiIn: MidiReader, // TODO: relative timing makes more sense.
    MidiOut: MidiWriter,
    F: Zero,
    R: AudioRenderer<F> + EventHandler<Timed<RawMidiEvent>>,
{
    assert!(buffer_size > 0);
    let number_of_channels = audio_in.number_of_channels();

    assert!(number_of_channels > 0);
    let input_buffers = create_buffers(number_of_channels, buffer_size);
    let mut output_buffers = create_buffers(number_of_channels, buffer_size);

    let mut event_queue = EventQueue::new(event_queue_capacity);

    while let Some(event) = midi_in.read_event() {
        let time = event.time_in_frames;
        event_queue.queue_event(event);
        if time as usize > buffer_size {
            break;
        }
    }

    loop {
        let input: Vec<&[F]> = input_buffers.iter().map(|b| b.as_slice()).collect();
        let mut output: Vec<&mut [F]> = output_buffers
            .iter_mut()
            .map(|b| b.as_mut_slice())
            .collect();
        for event_index in 0..event_queue.len() {
            plugin.handle_event(event_queue[event_index]);
        }
        plugin.render_buffer(input.as_slice(), output.as_mut_slice());
    }
}
