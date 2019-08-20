use crate::dev_utilities::chunk::{buffers_as_mut_slice, buffers_as_slice, AudioBuffer};
use crate::event::{EventHandler, RawMidiEvent, Timed};
use crate::AudioRenderer;
use num_traits::Zero;
use std::fmt::Debug;

pub mod dummy;
#[cfg(feature = "backend-file-hound")]
pub mod hound;
pub mod memory;
#[cfg(feature = "backend-file-rimd")]
pub mod rimd; // TODO: choose better naming.

pub trait AudioReader<F> {
    fn number_of_channels(&self) -> usize;
    fn frames_per_second(&self) -> u64;
    fn frames_per_microsecond(&self) -> u64 {
        self.frames_per_second() * MICROSECONDS_PER_SECOND
    }

    /// Fill the buffers. Return the number of frames that have been written.
    /// If it is `<` the number of frames in the input, now more frames can be expected.
    fn fill_buffer(&mut self, output: &mut [&mut [F]]) -> usize;
}

pub trait AudioWriter<F> {
    // TODO: This does not foresee error handling in any way ...
    // TODO: What if the writer gets an unexpected number of channels?
    fn write_buffer(&mut self, buffer: &[&[F]]);
}

pub const MICROSECONDS_PER_SECOND: u64 = 1_000_000;

pub struct DeltaEvent<E> {
    microseconds_since_previous_event: u64,
    event: E,
}

pub trait MidiReader {
    fn read_event(&mut self) -> Option<DeltaEvent<RawMidiEvent>>;
}

pub trait MidiWriter {
    fn write_event(&mut self, event: DeltaEvent<RawMidiEvent>);
}

pub fn run<F, AudioIn, AudioOut, MidiIn, MidiOut, R>(
    mut plugin: R,
    buffer_size_in_frames: usize,
    mut audio_in: AudioIn,
    mut audio_out: AudioOut,
    mut midi_in: MidiIn,
    mut midi_out: MidiOut,
) where
    AudioIn: AudioReader<F>,
    AudioOut: AudioWriter<F>,
    MidiIn: MidiReader,
    MidiOut: MidiWriter,
    F: Zero,
    R: AudioRenderer<F> + EventHandler<Timed<RawMidiEvent>>,
{
    assert!(buffer_size_in_frames > 0);
    assert!(buffer_size_in_frames < u32::max_value() as usize);

    let number_of_channels = audio_in.number_of_channels();
    assert!(number_of_channels > 0);

    let frames_per_second = audio_in.frames_per_second();
    assert!(frames_per_second > 0);

    let mut input_buffers = AudioBuffer::zero(number_of_channels, buffer_size_in_frames).inner();
    let mut output_buffers = AudioBuffer::zero(number_of_channels, buffer_size_in_frames).inner();

    let mut spare_event = None;
    let mut last_time_in_frames = 0;
    let mut last_event_time_in_microseconds = 0;

    let frames_per_microsecond = audio_in.frames_per_microsecond();

    loop {
        // Read audio.
        let frames_read = audio_in.fill_buffer(&mut buffers_as_mut_slice(
            &mut input_buffers,
            buffer_size_in_frames,
        ));
        assert!(frames_read <= buffer_size_in_frames);
        if frames_read == 0 {
            break;
        }

        // Handle events
        if let Some(leftover) = spare_event.take() {
            plugin.handle_event(Timed {
                time_in_frames: (last_event_time_in_microseconds / frames_per_microsecond
                    - last_time_in_frames) as u32,
                event: leftover,
            });
        }
        while let Some(event) = midi_in.read_event() {
            last_event_time_in_microseconds += event.microseconds_since_previous_event;
            let time_in_frames =
                last_event_time_in_microseconds / frames_per_microsecond - last_time_in_frames;
            if time_in_frames < buffer_size_in_frames as u64 {
                plugin.handle_event(Timed {
                    time_in_frames: time_in_frames as u32,
                    event: event.event,
                });
            } else {
                spare_event = Some(event.event);
                break;
            }
        }

        plugin.render_buffer(
            &buffers_as_slice(&input_buffers, frames_read),
            &mut buffers_as_mut_slice(&mut output_buffers, frames_read),
        );

        audio_out.write_buffer(&buffers_as_slice(&output_buffers, frames_read));

        if frames_read < buffer_size_in_frames {
            break;
        }

        last_time_in_frames += buffer_size_in_frames as u64;
    }
}

#[cfg(test)]
struct TestReader<'b, F> {
    inner: memory::AudioBufferReader<'b, F>,
    expected_channels: usize,
    expected_buffer_sizes: Vec<usize>,
    number_of_calls_to_fill_buffer: usize,
}

#[cfg(test)]
impl<'b, F> TestReader<'b, F> {
    fn new(
        reader: memory::AudioBufferReader<'b, F>,
        expected_channels: usize,
        expected_buffer_sizes: Vec<usize>,
    ) -> Self {
        Self {
            inner: reader,
            expected_channels,
            expected_buffer_sizes,
            number_of_calls_to_fill_buffer: 0,
        }
    }
}

#[cfg(test)]
impl<'b, F> AudioReader<F> for TestReader<'b, F>
where
    F: Copy,
{
    fn number_of_channels(&self) -> usize {
        self.inner.number_of_channels()
    }

    fn frames_per_second(&self) -> u64 {
        self.inner.frames_per_second()
    }

    fn fill_buffer(&mut self, output: &mut [&mut [F]]) -> usize {
        assert_eq!(output.len(), self.expected_channels);
        for channel in output.iter() {
            assert_eq!(
                self.expected_buffer_sizes[self.number_of_calls_to_fill_buffer],
                channel.len()
            )
        }
        self.number_of_calls_to_fill_buffer += 1;
        self.inner.fill_buffer(output)
    }
}

pub struct TestWriter<'w, T, F>
where
    T: AudioWriter<F>,
{
    inner: &'w mut T,
    expected_chunks: Vec<AudioBuffer<F>>,
    chunk_index: usize,
}

impl<'w, T, F> TestWriter<'w, T, F>
where
    T: AudioWriter<F>,
{
    pub fn new(writer: &'w mut T, expected_chunks: Vec<AudioBuffer<F>>) -> Self {
        Self {
            inner: writer,
            expected_chunks,
            chunk_index: 0,
        }
    }
}

impl<'w, T, F> AudioWriter<F> for TestWriter<'w, T, F>
where
    T: AudioWriter<F>,
    F: Debug + PartialEq,
{
    fn write_buffer(&mut self, chunk: &[&[F]]) {
        assert!(self.chunk_index < self.expected_chunks.len());
        let expected_chunk = &self.expected_chunks[self.chunk_index];
        assert_eq!(chunk, expected_chunk.as_slices().as_slice());
        self.inner.write_buffer(chunk);
        self.chunk_index += 1;
    }
}

#[cfg(test)]
mod tests {
    mod run {
        use super::super::{TestReader, TestWriter};
        use crate::backend::file_backend::dummy::{AudioDummy, MidiDummy};
        use crate::backend::file_backend::memory::{AudioBufferReader, AudioBufferWriter};
        use crate::dev_utilities::{chunk::AudioBuffer, TestPlugin};
        use crate::event::EventHandler;
        use crate::{AudioRenderer, AudioRendererMeta};

        struct DummyMeta;

        const EXPECTED_SAMPLE_RATE: f64 = 1234.0;
        impl AudioRendererMeta for DummyMeta {
            const MAX_NUMBER_OF_AUDIO_INPUTS: usize = 2;
            const MAX_NUMBER_OF_AUDIO_OUTPUTS: usize = 2;

            fn set_sample_rate(&mut self, sample_rate: f64) {
                assert_eq!(sample_rate, EXPECTED_SAMPLE_RATE);
            }
        }

        #[test]
        fn copies_input_buffer_to_output_buffer() {
            let buffer_size = 2;
            let input_data = audio_buffer![[1, 2, 3, 4, 5, 6, 7], [8, 9, 10, 11, 12, 13, 14]];
            let output_data = audio_buffer![
                [-1, -2, -3, -4, -5, -6, -7],
                [-8, -9, -10, -11, -12, -13, -14]
            ];
            let test_plugin = TestPlugin::new(
                input_data.clone().split(buffer_size),
                output_data.clone().split(buffer_size),
                vec![vec![], vec![], vec![], vec![]],
                DummyMeta,
            );
            let mut output_buffer = AudioBuffer::new(2);
            super::super::run(
                test_plugin,
                2,
                TestReader::new(
                    AudioBufferReader::new(&input_data, EXPECTED_SAMPLE_RATE as u64),
                    2,
                    vec![buffer_size; 4],
                ),
                TestWriter::new(
                    &mut AudioBufferWriter::new(&mut output_buffer),
                    output_data.clone().split(buffer_size),
                ),
                MidiDummy::new(),
                MidiDummy::new(),
            );
            assert_eq!(output_buffer, output_data);
        }
    }
}
