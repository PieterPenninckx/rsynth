//! Utilities for testing.

use crate::buffer::{AudioBufferInOut, AudioChunk};
use crate::event::{ContextualEventHandler, EventHandler};
use crate::{AudioHandler, AudioHandlerMeta, ContextualAudioRenderer};
use std::fmt::Debug;

pub struct DummyEventHandler;

impl<E> EventHandler<E> for DummyEventHandler {
    fn handle_event(&mut self, _event: E) {}
}

impl<E, C> ContextualEventHandler<E, C> for DummyEventHandler {
    fn handle_event(&mut self, _event: E, _context: &mut C) {}
}

/// A plugin useful for writing automated tests.
// TODO: Add more documentation.
pub struct TestPlugin<S, E, M> {
    expected_inputs: Vec<AudioChunk<S>>,
    provided_outputs: Vec<AudioChunk<S>>,
    expected_events: Vec<Vec<E>>,
    provided_events: Vec<Vec<E>>,
    meta: M,
    buffer_index: usize,
    event_index: usize,
}

impl<S, E, M> TestPlugin<S, E, M> {
    pub fn new(
        expected_inputs: Vec<AudioChunk<S>>,
        provided_outputs: Vec<AudioChunk<S>>,
        expected_events: Vec<Vec<E>>,
        provided_events: Vec<Vec<E>>,
        meta: M,
    ) -> Self {
        assert_eq!(expected_inputs.len(), provided_outputs.len(), "When constructing test plugin, `expected_inputs`, `provided_outputs`, `expected_events` and `provided_events` should all have the same length.");
        assert_eq!(expected_inputs.len(), expected_events.len(), "When constructing test plugin, `expected_inputs`, `provided_outputs`, `expected_events` and `provided_events` should all have the same length.");
        assert_eq!(expected_inputs.len(), provided_events.len(), "When constructing test plugin, `expected_inputs`, `provided_outputs`, `expected_events` and `provided_events` should all have the same length.");
        TestPlugin {
            expected_inputs,
            provided_outputs,
            expected_events,
            provided_events,
            meta,
            buffer_index: 0,
            event_index: 0,
        }
    }

    pub fn check_last(&self) {
        assert_eq!(self.buffer_index, self.expected_inputs.len());
    }
}

impl<S, E, M> AudioHandlerMeta for TestPlugin<S, E, M>
where
    M: AudioHandlerMeta,
{
    fn max_number_of_audio_inputs(&self) -> usize {
        self.meta.max_number_of_audio_inputs()
    }
    fn max_number_of_audio_outputs(&self) -> usize {
        self.meta.max_number_of_audio_outputs()
    }
}

impl<S, E, M> AudioHandler for TestPlugin<S, E, M>
where
    M: AudioHandler,
{
    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.meta.set_sample_rate(sample_rate);
    }
}

impl<S, E, M, C> ContextualAudioRenderer<S, C> for TestPlugin<S, E, M>
where
    S: PartialEq + Debug + Copy,
    C: EventHandler<E>,
{
    fn render_buffer(&mut self, buffer: &mut AudioBufferInOut<S>, context: &mut C) {
        assert!(
            self.buffer_index < self.expected_inputs.len(),
            "`render_buffer` called more often than expected: expected only {} times",
            self.expected_inputs.len()
        );

        assert_eq!(
            self.event_index,
            self.expected_events[self.buffer_index].len(),
            "`handle_event` called {} times for buffer {} (zero-based), but {} times was expected",
            self.event_index,
            self.buffer_index,
            self.expected_events[self.buffer_index].len()
        );
        self.event_index = 0;

        let expected_input_channels = &self.expected_inputs[self.buffer_index].channels();
        assert_eq!(
            buffer.number_of_input_channels(),
            expected_input_channels.len(),
            "`render_buffer` called with {} input channels, but {} were expected",
            buffer.number_of_input_channels(),
            expected_input_channels.len()
        );
        for (input_channel_index, input_channel) in buffer.inputs().channels().iter().enumerate() {
            let expected_input_channel = &expected_input_channels[input_channel_index];
            assert_eq!(
                input_channel.len(),
                expected_input_channel.len(),
                "mismatch in input channel #{} in buffer #{}: \
                 expected input channel with length {}, but got one with length {}",
                input_channel_index,
                self.buffer_index,
                input_channel.len(),
                expected_input_channel.len()
            );
            for (sample_index, sample) in input_channel.iter().enumerate() {
                assert_eq!(
                    *sample,
                    expected_input_channel[sample_index],
                    "mismatch in input sample with index #{} in channel #{} in buffer #{}: \
                     expected {:?} but got {:?}",
                    sample_index,
                    input_channel_index,
                    self.buffer_index,
                    expected_input_channel[sample_index],
                    sample
                );
            }
        }

        let expected_output_channels = self.provided_outputs[self.buffer_index].channels();
        assert_eq!(
            buffer.number_of_output_channels(),
            expected_output_channels.len()
        );
        // TODO: Use an iterator here.
        for output_channel_index in 0..expected_output_channels.len() {
            let expected_output_channel = &expected_output_channels[output_channel_index];
            assert_eq!(
                buffer.number_of_frames(),
                expected_output_channel.len(),
                "mismatch in output channel #{} in buffer #{}: \
                 expected one with length {}, but got one with length {}",
                output_channel_index,
                self.buffer_index,
                expected_output_channel.len(),
                buffer.number_of_frames()
            );
            buffer
                .outputs()
                .index_channel(output_channel_index)
                .copy_from_slice(expected_output_channel);
        }

        todo!();
        /*
        for (output_channel_index, output_channel) in outputs.iter_mut().enumerate() {
            let expected_output_channel = &expected_output_channels[output_channel_index];
            assert_eq!(
                output_channel.len(),
                expected_output_channel.len(),
                "mismatch in output channel #{} in buffer #{}: \
                 expected one with length {}, but got one with length {}",
                output_channel_index,
                self.buffer_index,
                expected_output_channel.len(),
                output_channel.len()
            );
            output_channel.copy_from_slice(expected_output_channel);
        }
        */
        let events = self.provided_events.drain(..1).next().unwrap();
        for event in events {
            context.handle_event(event);
        }

        self.buffer_index += 1;
        self.event_index = 0;
    }
}

impl<S, E, M> EventHandler<E> for TestPlugin<S, E, M>
where
    E: PartialEq + Debug,
{
    fn handle_event(&mut self, event: E) {
        assert!(
            self.buffer_index < self.expected_events.len(),
            "`handle_event` is called after {} calls to `render_buffer`; this is unexpected",
            self.expected_events.len()
        );
        dbg!(&self.expected_events);
        let expected_events_for_this_buffer = &self.expected_events[self.buffer_index];
        assert!(
            self.event_index < expected_events_for_this_buffer.len(),
            "`handle_events` is called more than {0} times after {1} calls to `render_buffer`; \
             only {0} times are expected because we expect only \
             {0} events for the subsequent buffer",
            expected_events_for_this_buffer.len(),
            self.buffer_index
        );
        assert_eq!(
            event,
            expected_events_for_this_buffer[self.event_index],
            "mismatch for event #{} after {} calls to `render_buffer`: \
             expected {:?} but got {:?}.",
            self.event_index,
            self.buffer_index,
            expected_events_for_this_buffer[self.event_index],
            event
        );
        self.event_index += 1;
    }
}
