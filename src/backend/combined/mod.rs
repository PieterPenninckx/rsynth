//! Combine different back-ends for audio input, audio output, midi input and
//! midi output, mostly for offline rendering and testing.
//!
//! Support is only enabled if `rsynth` is compiled with the "backend-combined"
//! feature, see [the cargo reference] for more information on setting cargo features.
//!
//! The [`run`] function can be used to run a plugin and read audio and midi from the
//! inputs and write audio and midi to the outputs.
//!
//! Currently, the following inputs and outputs are available:
//!
//! * Dummy: [`AudioDummy`]: dummy audio input (generates silence) and output and [`MidiDummy`]: dummy midi input (generates no events) and output
//! * Hound: [`HoundAudioReader`] and [`HoundAudioWriter`]: read and write `.wav` files (behind the "backend-combined-hound" feature)
//! * Rimd: [`RimdMidiReader`] and [`RimdMidiWriter`]: reand and write `.mid` files (behind the "backend-combined-rimd" feature)
//! * Memory: [`AudioBufferReader`] and [`AudioBufferWriter`]: read and write audio from memory
//! * Testing: [`TestAudioReader`] and [`TestAudioWriter`]: audio input and output, to be used in tests
//!
//! [`AudioDummy`]: ./dummy/struct.AudioDummy.html
//! [`MidiDummy`]: ./dummy/struct.MidiDummy.html
//! [`HoundAudioReader`]: ./hound/struct.HoundAudioReader.html
//! [`HoundAudioWriter`]: ./hound/struct.HoundAudioWriter.html
//! [`RimdMidiReader`]: ./rimd/struct.RimdMidiReader.html
//! [`RimdMidiWriter`]: ./rimd/struct.RimdMidiWriter.html
//! [`TestAudioReader`]: ./struct.TestAudioReader.html
//! [`TestAudioWriter`]: ./struct.TestAudioWriter.html
//! [`AudioBufferReader`]: ./memory/struct.AudioBufferReader.html
//! [`AudioBufferWriter`]: ./memory/struct.AudioBufferWriter.html
//! [`run`]: ./fn.run.html
//! [the cargo reference]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-features-section

use crate::buffer::{
    buffers_as_mut_slice, buffers_as_slice, AudioBufferIn, AudioBufferInOut, AudioBufferOut,
    AudioChunk,
};
use crate::event::event_queue::{AlwaysInsertNewAfterOld, EventQueue};
use crate::event::{DeltaEvent, EventHandler, RawMidiEvent, Timed};
use crate::ContextualAudioRenderer;
use num_traits::Zero;
use std::fmt::Debug;
use vecstorage::VecStorage;

pub mod dummy;
#[cfg(feature = "backend-combined-hound")]
pub mod hound;
pub mod memory;
#[cfg(feature = "backend-combined-rimd")]
pub mod rimd; // TODO: choose better name for this module.

/// Define how audio is read.
///
/// This trait is generic over `S`, which represents the data-type used for a sample.
pub trait AudioReader<S>
where
    S: Copy,
{
    /// The type of the error that occurs when reading data.
    type Err;

    /// The number of audio channels that can be read.
    fn number_of_channels(&self) -> usize;

    /// The sampling frequency in frames per second.
    fn frames_per_second(&self) -> u64;

    /// Fill the buffers. Return the number of frames that have been read and written
    /// to the buffer.
    /// If the return value is `<` the number of frames in the input, no more frames can be expected.
    fn fill_buffer(&mut self, output: &mut AudioBufferOut<S>) -> Result<usize, Self::Err>;
}

/// Define how audio is written.
///
/// This trait is generic over `S`, which represents the data-type used for a sample.
pub trait AudioWriter<S>
where
    S: Copy,
{
    /// The type of the error that occurs when reading data.
    type Err;
    // TODO: What if the writer gets an unexpected number of channels?
    fn write_buffer(&mut self, buffer: &AudioBufferIn<S>) -> Result<(), Self::Err>;
}

pub const MICROSECONDS_PER_SECOND: u64 = 1_000_000;

pub trait MidiWriter {
    fn write_event(&mut self, event: DeltaEvent<RawMidiEvent>);
}

// TODO: find a better name for this.
pub struct MidiWriterWrapper<W>
where
    W: MidiWriter,
{
    inner: W,
    current_time_in_frames: u64,
    previous_time_in_microseconds: u64,
    micro_seconds_per_frame: f64,
    event_queue: EventQueue<RawMidiEvent>,
}

impl<W> MidiWriterWrapper<W>
where
    W: MidiWriter,
{
    pub fn new(inner: W, micro_seconds_per_frame: f64) -> Self {
        MidiWriterWrapper {
            inner,
            previous_time_in_microseconds: 0,
            current_time_in_frames: 0,
            micro_seconds_per_frame,
            event_queue: EventQueue::new(1024),
        }
    }

    pub fn step_frames(&mut self, number_of_frames: u64) {
        for event in self.event_queue.iter() {
            let current_time_in_frames =
                self.current_time_in_frames + (event.time_in_frames as u64);
            let current_time_in_microseconds =
                (current_time_in_frames as f64 * self.micro_seconds_per_frame) as u64;
            let delta_event = DeltaEvent {
                microseconds_since_previous_event: current_time_in_microseconds
                    - self.previous_time_in_microseconds,
                event: event.event,
            };
            self.inner.write_event(delta_event);
            self.previous_time_in_microseconds = current_time_in_microseconds;
        }
        self.event_queue.clear();
        self.current_time_in_frames += number_of_frames;
    }
}

impl<W> EventHandler<Timed<RawMidiEvent>> for MidiWriterWrapper<W>
where
    W: MidiWriter,
{
    fn handle_event(&mut self, event: Timed<RawMidiEvent>) {
        self.event_queue.queue_event(event, AlwaysInsertNewAfterOld);
    }
}

/// The error type that represents the errors you can get from the [`run`] function.
///
/// [`run`]: ./fn.run.html
#[derive(Debug)]
pub enum CombinedError<AudioInErr, AudioOutErr> {
    /// An error occurred when reading the audio.
    AudioInError(AudioInErr),
    /// An error occurred when writing the audio.
    AudioOutError(AudioOutErr),
}

/// Run an audio renderer with the given audio input, audio output, midi input and midi output.
///
/// Parameters
/// ==========
/// * `buffer_size_in_frames`: the buffer size in frames.
///
/// Panics
/// ======
/// Panics if `buffer_size_in_frames` is `0` or `> u32::max_value()`.
// TODO: support different number of input and output channels.
pub fn run<S, AudioIn, AudioOut, MidiIn, MidiOut, R>(
    plugin: &mut R,
    buffer_size_in_frames: usize,
    mut audio_in: AudioIn,
    mut audio_out: AudioOut,
    midi_in: MidiIn,
    midi_out: MidiOut,
) -> Result<(), CombinedError<<AudioIn as AudioReader<S>>::Err, <AudioOut as AudioWriter<S>>::Err>>
where
    AudioIn: AudioReader<S>,
    AudioOut: AudioWriter<S>,
    MidiIn: Iterator<Item = DeltaEvent<RawMidiEvent>>,
    MidiOut: MidiWriter,
    S: Copy + Zero + 'static,
    R: ContextualAudioRenderer<S, MidiWriterWrapper<MidiOut>> + EventHandler<Timed<RawMidiEvent>>,
{
    assert!(buffer_size_in_frames > 0);
    assert!(buffer_size_in_frames < u32::max_value() as usize);

    let number_of_input_channels = audio_in.number_of_channels();

    let frames_per_second = audio_in.frames_per_second();
    assert!(frames_per_second > 0);

    let mut input_buffers =
        AudioChunk::zero(number_of_input_channels, buffer_size_in_frames).inner();
    let mut output_buffers =
        AudioChunk::zero(number_of_input_channels, buffer_size_in_frames).inner();

    let mut last_time_in_frames = 0;
    let mut last_event_time_in_microseconds = 0;

    let frames_per_second = audio_in.frames_per_second();

    let mut writer = MidiWriterWrapper::new(
        midi_out,
        MICROSECONDS_PER_SECOND as f64 / frames_per_second as f64,
    );

    let mut peekable_midi_reader = midi_in.peekable();

    let mut conversion_storage: VecStorage<&'static [S]> =
        VecStorage::with_capacity(number_of_input_channels);

    loop {
        let mut slices = buffers_as_mut_slice(&mut input_buffers, buffer_size_in_frames);
        let mut buffer = AudioBufferOut::new(&mut slices, buffer_size_in_frames);
        // Read audio.
        let frames_read = match audio_in.fill_buffer(&mut buffer) {
            Ok(f) => f,
            Err(e) => {
                return Err(CombinedError::AudioInError(e));
            }
        };
        assert!(frames_read <= buffer_size_in_frames);
        if frames_read == 0 {
            break;
        }

        // Handle events
        if let Some(event) = peekable_midi_reader.peek() {
            let time_in_frames = (last_event_time_in_microseconds
                + event.microseconds_since_previous_event)
                * frames_per_second
                / MICROSECONDS_PER_SECOND
                - last_time_in_frames;
            if time_in_frames < buffer_size_in_frames as u64 {
                let event = peekable_midi_reader
                    .next()
                    .expect("to see event that I just peeked at");
                plugin.handle_event(Timed {
                    time_in_frames: time_in_frames as u32,
                    event: event.event,
                });
                last_event_time_in_microseconds += event.microseconds_since_previous_event;
            }
        }

        let inputs = buffers_as_slice(&input_buffers, frames_read);
        let mut outputs = buffers_as_mut_slice(&mut output_buffers, frames_read);
        let mut buffer = AudioBufferInOut::new(&inputs, &mut outputs, frames_read);
        plugin.render_buffer(&mut buffer, &mut writer);

        let mut guard = conversion_storage.vec_guard();
        let converted = buffer.outputs().as_audio_buffer_in(&mut guard);

        if let Err(e) = audio_out.write_buffer(&converted) {
            return Err(CombinedError::AudioOutError(e));
        }

        writer.step_frames(frames_read as u64);

        if frames_read < buffer_size_in_frames {
            break;
        }

        last_time_in_frames += buffer_size_in_frames as u64;
    }
    Ok(())
}

pub struct TestAudioReader<'b, S>
where
    S: Copy,
{
    inner: memory::AudioBufferReader<'b, S>,
    expected_channels: usize,
    expected_buffer_sizes: Vec<usize>,
    number_of_calls_to_fill_buffer: usize,
}

impl<'b, S> TestAudioReader<'b, S>
where
    S: Copy,
{
    fn new(
        reader: memory::AudioBufferReader<'b, S>,
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

impl<'b, S> AudioReader<S> for TestAudioReader<'b, S>
where
    S: Copy,
{
    type Err = std::convert::Infallible;

    fn number_of_channels(&self) -> usize {
        self.inner.number_of_channels()
    }

    fn frames_per_second(&self) -> u64 {
        self.inner.frames_per_second()
    }

    fn fill_buffer(&mut self, output: &mut AudioBufferOut<S>) -> Result<usize, Self::Err> {
        assert_eq!(output.number_of_channels(), self.expected_channels);
        assert_eq!(
            self.expected_buffer_sizes[dbg!(self.number_of_calls_to_fill_buffer)],
            output.number_of_frames()
        );
        self.number_of_calls_to_fill_buffer += 1;
        self.inner.fill_buffer(output)
    }
}

pub struct TestAudioWriter<'w, T, S>
where
    T: AudioWriter<S>,
    S: Copy,
{
    inner: &'w mut T,
    expected_chunks: Vec<AudioChunk<S>>,
    chunk_index: usize,
}

impl<'w, T, S> TestAudioWriter<'w, T, S>
where
    T: AudioWriter<S>,
    S: Copy,
{
    pub fn new(writer: &'w mut T, expected_chunks: Vec<AudioChunk<S>>) -> Self {
        Self {
            inner: writer,
            expected_chunks,
            chunk_index: 0,
        }
    }
}

impl<'w, T, S> AudioWriter<S> for TestAudioWriter<'w, T, S>
where
    T: AudioWriter<S>,
    S: Debug + PartialEq + Copy,
{
    type Err = <T as AudioWriter<S>>::Err;

    fn write_buffer(&mut self, chunk: &AudioBufferIn<S>) -> Result<(), Self::Err> {
        assert!(self.chunk_index < self.expected_chunks.len());
        let expected_chunk = &self.expected_chunks[self.chunk_index];
        assert_eq!(chunk.channels(), expected_chunk.as_slices().as_slice());
        self.inner.write_buffer(chunk)?;
        self.chunk_index += 1;
        Ok(())
    }
}

pub struct TestMidiReader {
    provided_events: Vec<DeltaEvent<RawMidiEvent>>,
    event_index: usize,
}

impl TestMidiReader {
    pub fn new(provided_events: Vec<DeltaEvent<RawMidiEvent>>) -> Self {
        TestMidiReader {
            provided_events,
            event_index: 0,
        }
    }
}

impl Iterator for TestMidiReader {
    type Item = DeltaEvent<RawMidiEvent>;
    fn next(&mut self) -> Option<DeltaEvent<RawMidiEvent>> {
        if self.event_index < self.provided_events.len() {
            let result = self.provided_events[self.event_index];
            self.event_index += 1;
            Some(result)
        } else {
            None
        }
    }
}

pub struct TestMidiWriter {
    expected_events: Vec<DeltaEvent<RawMidiEvent>>,
    event_index: usize,
}

impl TestMidiWriter {
    pub fn new(expected_events: Vec<DeltaEvent<RawMidiEvent>>) -> Self {
        TestMidiWriter {
            expected_events,
            event_index: 0,
        }
    }

    pub fn check_last(&self) {
        assert_eq!(self.event_index, self.expected_events.len());
    }
}

impl TestMidiWriter {}

impl MidiWriter for TestMidiWriter {
    fn write_event(&mut self, event: DeltaEvent<RawMidiEvent>) {
        assert!(
            self.event_index < self.expected_events.len(),
            "Only {} events are expected, but {} events are written.",
            self.expected_events.len(),
            self.event_index + 1
        );
        assert_eq!(self.expected_events[self.event_index], event);
        self.event_index += 1;
    }
}

#[cfg(test)]
mod tests {
    mod run {
        use super::super::{
            dummy::MidiDummy,
            memory::{AudioBufferReader, AudioBufferWriter},
            DeltaEvent, TestAudioReader, TestAudioWriter,
        };
        use crate::backend::combined::{TestMidiReader, TestMidiWriter};
        use crate::buffer::AudioChunk;
        use crate::event::{RawMidiEvent, Timed};
        use crate::test_utilities::TestPlugin;
        use crate::{AudioHandler, AudioHandlerMeta};

        struct DummyMeta;

        const EXPECTED_SAMPLE_RATE: f64 = 1234.0;
        impl AudioHandlerMeta for DummyMeta {
            fn max_number_of_audio_inputs(&self) -> usize {
                2
            }
            fn max_number_of_audio_outputs(&self) -> usize {
                2
            }
        }

        impl AudioHandler for DummyMeta {
            fn set_sample_rate(&mut self, sample_rate: f64) {
                assert_eq!(sample_rate, EXPECTED_SAMPLE_RATE);
            }
        }

        #[test]
        fn reads_events_at_the_right_time() {
            const BUFFER_SIZE: usize = 3;
            const NUMBER_OF_CHANNELS: usize = 1;
            const SAMPLE_RATE: u64 = 8000;
            let input_data = AudioChunk::<i16>::zero(1, 16);
            let output_data = AudioChunk::<i16>::zero(1, 16);

            // So 1 frame  is 1/8000 seconds,
            //    8 frames is 1/1000 seconds = 1ms = 1000 microsecond.
            let event = RawMidiEvent::new(&[1, 2, 3]);
            // Event is expected at frame 8:
            // 0 1 2 3 4 5 6 7 8        (in 1000 microseconds)
            // . . .|. . .|. . E|. . .|. . .|.
            let input_event = DeltaEvent {
                microseconds_since_previous_event: 1000,
                event,
            };

            let mut test_plugin = TestPlugin::new(
                input_data.clone().split(BUFFER_SIZE),
                output_data.clone().split(BUFFER_SIZE),
                vec![
                    vec![],
                    vec![],
                    vec![Timed::new(2, event)],
                    vec![],
                    vec![],
                    vec![],
                ],
                vec![Vec::new(); 6],
                DummyMeta,
            );
            let mut output_buffer = AudioChunk::new(NUMBER_OF_CHANNELS);
            super::super::run(
                &mut test_plugin,
                BUFFER_SIZE,
                TestAudioReader::new(
                    AudioBufferReader::new(&input_data, SAMPLE_RATE),
                    NUMBER_OF_CHANNELS,
                    vec![
                        BUFFER_SIZE,
                        BUFFER_SIZE,
                        BUFFER_SIZE,
                        BUFFER_SIZE,
                        BUFFER_SIZE,
                        BUFFER_SIZE,
                    ],
                ),
                TestAudioWriter::new(
                    &mut AudioBufferWriter::new(&mut output_buffer),
                    output_data.clone().split(BUFFER_SIZE),
                ),
                TestMidiReader::new(vec![input_event]),
                MidiDummy::new(),
            )
            .expect("Unexpected error");
            test_plugin.check_last();
        }

        #[test]
        fn copies_input_buffer_to_output_buffer() {
            let buffer_size = 2;
            let input_data = audio_chunk![[1, 2, 3, 4, 5, 6, 7], [8, 9, 10, 11, 12, 13, 14]];
            let output_data = audio_chunk![
                [-1, -2, -3, -4, -5, -6, -7],
                [-8, -9, -10, -11, -12, -13, -14]
            ];
            let mut test_plugin = TestPlugin::new(
                input_data.clone().split(buffer_size),
                output_data.clone().split(buffer_size),
                vec![vec![], vec![], vec![], vec![]],
                vec![Vec::new(); 4],
                DummyMeta,
            );
            let mut output_buffer = AudioChunk::new(2);
            super::super::run(
                &mut test_plugin,
                2,
                TestAudioReader::new(
                    AudioBufferReader::new(&input_data, EXPECTED_SAMPLE_RATE as u64),
                    2,
                    vec![buffer_size; 4],
                ),
                TestAudioWriter::new(
                    &mut AudioBufferWriter::new(&mut output_buffer),
                    output_data.clone().split(buffer_size),
                ),
                MidiDummy::new(),
                MidiDummy::new(),
            )
            .expect("Unexpected error.");
            assert_eq!(output_buffer, output_data);
        }

        #[test]
        fn writes_events_at_the_right_time() {
            const BUFFER_SIZE: usize = 3;
            const NUMBER_OF_CHANNELS: usize = 1;
            const SAMPLE_RATE: u64 = 8000;
            let input_data = AudioChunk::<i16>::zero(1, 16);
            let output_data = AudioChunk::<i16>::zero(1, 16);

            // So 1 frame  is 1/8000 seconds,
            //    8 frames is 1/1000 seconds = 1ms = 1000 microsecond.
            let event = RawMidiEvent::new(&[1, 2, 3]);
            let input_event = DeltaEvent {
                microseconds_since_previous_event: 1000,
                event,
            };
            // Event is created at frame 8:
            // 0 1 2 3 4 5 6 7 8        (in 1000 microseconds)
            // . . .|. . .|. . E|. . .|. . .|.

            let mut test_plugin = TestPlugin::new(
                input_data.clone().split(BUFFER_SIZE),
                output_data.clone().split(BUFFER_SIZE),
                vec![Vec::new(); 6],
                vec![
                    vec![],
                    vec![],
                    vec![Timed::new(2, event)],
                    vec![],
                    vec![],
                    vec![],
                ],
                DummyMeta,
            );
            let mut output_buffer = AudioChunk::new(NUMBER_OF_CHANNELS);
            super::super::run(
                &mut test_plugin,
                BUFFER_SIZE,
                TestAudioReader::new(
                    AudioBufferReader::new(&input_data, SAMPLE_RATE),
                    NUMBER_OF_CHANNELS,
                    vec![
                        BUFFER_SIZE,
                        BUFFER_SIZE,
                        BUFFER_SIZE,
                        BUFFER_SIZE,
                        BUFFER_SIZE,
                        BUFFER_SIZE,
                    ],
                ),
                TestAudioWriter::new(
                    &mut AudioBufferWriter::new(&mut output_buffer),
                    output_data.clone().split(BUFFER_SIZE),
                ),
                MidiDummy::new(),
                TestMidiWriter::new(vec![input_event]),
            );
        }

        #[test]
        fn writes_two_events_at_the_right_time() {
            const BUFFER_SIZE: usize = 3;
            const NUMBER_OF_CHANNELS: usize = 1;
            const SAMPLE_RATE: u64 = 8000;
            let input_data = AudioChunk::<i16>::zero(1, 16);
            let output_data = AudioChunk::<i16>::zero(1, 16);

            // So 1 frame  is 1/8000 seconds,
            //    2 frames is 1/4000 seconds = 0.24 ms = 250 microsecond
            //    8 frames is 1/1000 seconds = 1ms = 1000 microsecond.
            let event1 = RawMidiEvent::new(&[1, 2, 3]);
            let output_event1 = DeltaEvent {
                microseconds_since_previous_event: 1000,
                event: event1,
            };
            let event2 = RawMidiEvent::new(&[4, 5, 6]);
            let output_event2 = DeltaEvent {
                microseconds_since_previous_event: 250,
                event: event2,
            };

            let mut test_plugin = TestPlugin::new(
                input_data.clone().split(BUFFER_SIZE),
                output_data.clone().split(BUFFER_SIZE),
                vec![Vec::new(); 6],
                vec![
                    vec![],
                    vec![],
                    vec![Timed::new(2, event1)],
                    vec![Timed::new(1, event2)],
                    vec![],
                    vec![],
                ],
                DummyMeta,
            );
            let mut output_buffer = AudioChunk::new(NUMBER_OF_CHANNELS);
            super::super::run(
                &mut test_plugin,
                BUFFER_SIZE,
                TestAudioReader::new(
                    AudioBufferReader::new(&input_data, SAMPLE_RATE),
                    NUMBER_OF_CHANNELS,
                    vec![
                        BUFFER_SIZE,
                        BUFFER_SIZE,
                        BUFFER_SIZE,
                        BUFFER_SIZE,
                        BUFFER_SIZE,
                        BUFFER_SIZE,
                    ],
                ),
                TestAudioWriter::new(
                    &mut AudioBufferWriter::new(&mut output_buffer),
                    output_data.clone().split(BUFFER_SIZE),
                ),
                MidiDummy::new(),
                TestMidiWriter::new(vec![output_event1, output_event2]),
            )
            .expect("Unexpected error.");
        }

        #[test]
        fn writes_two_events_in_the_same_buffer_at_the_right_time() {
            const BUFFER_SIZE: usize = 3;
            const NUMBER_OF_CHANNELS: usize = 1;
            const SAMPLE_RATE: u64 = 8000;
            let input_data = AudioChunk::<i16>::zero(1, 16);
            let output_data = AudioChunk::<i16>::zero(1, 16);

            let event1 = RawMidiEvent::new(&[1, 2, 3]);
            let output_event1 = DeltaEvent {
                microseconds_since_previous_event: 750,
                event: event1,
            };
            let event2 = RawMidiEvent::new(&[4, 5, 6]);
            let output_event2 = DeltaEvent {
                microseconds_since_previous_event: 250,
                event: event2,
            };

            let mut test_plugin = TestPlugin::new(
                input_data.clone().split(BUFFER_SIZE),
                output_data.clone().split(BUFFER_SIZE),
                vec![Vec::new(); 6],
                vec![
                    vec![],
                    vec![],
                    vec![Timed::new(0, event1), Timed::new(2, event2)],
                    vec![],
                    vec![],
                    vec![],
                ],
                DummyMeta,
            );
            let mut output_buffer = AudioChunk::new(NUMBER_OF_CHANNELS);
            super::super::run(
                &mut test_plugin,
                BUFFER_SIZE,
                TestAudioReader::new(
                    AudioBufferReader::new(&input_data, SAMPLE_RATE),
                    NUMBER_OF_CHANNELS,
                    vec![
                        BUFFER_SIZE,
                        BUFFER_SIZE,
                        BUFFER_SIZE,
                        BUFFER_SIZE,
                        BUFFER_SIZE,
                        BUFFER_SIZE,
                    ],
                ),
                TestAudioWriter::new(
                    &mut AudioBufferWriter::new(&mut output_buffer),
                    output_data.clone().split(BUFFER_SIZE),
                ),
                MidiDummy::new(),
                TestMidiWriter::new(vec![output_event1, output_event2]),
            )
            .expect("Unexpected error.");
        }
    }
}
