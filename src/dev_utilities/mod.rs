//! Utilities to be used when developing backends and middleware.
//!
//! Writing a backend
//! =================
//!
//! Custom backends do not need to be in the `rsynth` crate, you can write
//! a backend in a separate crate. If you encounter problems that prevent you
//! from writing your backend in a separate crate (e.g., we have forgotten to
//! mark something as `pub`), let us know by opening an issue.
//!
//! Creating an input buffer and an output buffer
//! ---------------------------------------------
//!
//! When you pass `&[&[f32]]` for the input buffer and `&mut[&mut[f32]]`
//! for the output buffer, you may face the challenge that you can have
//! the buffers for each channel and you can `collect()` them into a `Vec`,
//! but you don't want to allocate that `Vec` in the real-time thread.
//! In order you to help overcome this problem, we provide
//! [`VecStorage` and `VecStorageMut`], which you can pre-allocate and re-use for every
//! call to `render_buffer` with different lifetimes of the slices.
//!
//! Writing custom events
//! ---------------------
//!
//! See ["Writing events" below].
//!
//! Publishing a backend crate
//! --------------------------
//!
//! When you publish a backend crate, let us know by opening an issue or pull request
//! so that we can link to it in the documentation of rsynth.
//!
//!
//! Writing events
//! ==============
//!
//! Implement `Copy` if possible
//! ----------------------------
//!
//! If possible, implement the `Copy` trait for the event,
//! so that the `Polyphonic` middleware can dispatch this event to all the voices.
//!
//!
//! [`VecStorage` and `VecStorageMut`]: ./vecstorage/index.html
//! ["Writing events" below]: ./index.html#writing-events
use crate::event::EventHandler;
use crate::{AudioRenderer, AudioRendererMeta};
use num_traits::Zero;
use std::fmt::Debug;
use std::mem;

pub mod vecstorage;

pub fn create_buffers<F: Zero>(number_of_channels: usize, buffer_size: usize) -> Vec<Vec<F>> {
    let mut buffers = Vec::with_capacity(number_of_channels);
    for _ in 0..number_of_channels {
        let mut buffer = Vec::with_capacity(buffer_size);
        for _ in 0..buffer_size {
            buffer.push(F::zero());
        }
        buffers.push(buffer);
    }
    buffers
}

// TODO: find a better name for this function.
pub fn slicify<'a, T>(vec: &'a Vec<Vec<T>>) -> Vec<&[T]> {
    vec.iter().map(|element| element.as_slice()).collect()
}

// TODO: find a better name for this function.
pub fn slicify_mut<'a, T>(vec: &'a mut Vec<Vec<T>>) -> Vec<&mut [T]> {
    vec.iter_mut()
        .map(|element| element.as_mut_slice())
        .collect()
}

/// Chunk buffers.
///
/// `channels` is a vector of channels, where each channel is a vector of samples.
/// This function returns a vector of chunks, where each chunk is a vector, holding for
/// each channel a vector of samples.
/// So
/// `[[1, 2, 3], [4, 5, 6]]` becomes `[[[1, 2], [4, 5]], [[3], [6]]]`.
pub fn chunk<T>(mut channels: Vec<Vec<T>>, chunk_size: usize) -> Vec<Vec<Vec<T>>> {
    assert!(chunk_size > 0);

    if channels.is_empty() {
        return Vec::new();
    }
    let number_of_samples = channels[0].len();
    assert!(channels
        .iter()
        .all(|channel| channel.len() == number_of_samples));

    let result_len = number_of_samples / chunk_size
        + if number_of_samples % chunk_size == 0 {
            0
        } else {
            1
        };

    let mut result = Vec::with_capacity(result_len);
    for _ in 0..result_len {
        result.push(Vec::new());
    }

    for mut channel in channels.drain(..) {
        let mut chunk_index = 0;
        let mut chunk = Vec::new();
        for sample in channel.drain(..) {
            chunk.push(sample);
            if chunk.len() == chunk_size {
                result[chunk_index].push(mem::replace(&mut chunk, Vec::new()));
                chunk_index += 1;
            }
        }
        if !chunk.is_empty() {
            result[chunk_index].push(chunk);
        }
    }
    result
}

#[test]
fn chunk_works_with_dividing_input_length() {
    let input = vec![vec![0, 1, 2, 3], vec![5, 6, 7, 8]];
    let observed = chunk(input, 2);
    assert_eq!(
        observed,
        vec![vec![vec![0, 1], vec![5, 6]], vec![vec![2, 3], vec![7, 8]]]
    )
}

#[test]
fn chunk_works_with_non_dividing_input_length() {
    let input = vec![vec![0, 1, 2, 3, 4], vec![5, 6, 7, 8, 9]];
    let observed = chunk(input, 2);
    assert_eq!(
        observed,
        vec![
            vec![vec![0, 1], vec![5, 6]],
            vec![vec![2, 3], vec![7, 8]],
            vec![vec![4], vec![9]]
        ]
    )
}

/// A plugin useful for writing automated tests.
pub struct TestPlugin<F, E, M: AudioRendererMeta> {
    expected_inputs: Vec<Vec<Vec<F>>>,
    provided_outputs: Vec<Vec<Vec<F>>>,
    expected_events: Vec<Vec<E>>,
    meta: M,
    buffer_index: usize,
    event_index: usize,
}

impl<F, E, M: AudioRendererMeta> TestPlugin<F, E, M> {
    pub fn new(
        expected_inputs: Vec<Vec<Vec<F>>>,
        provided_outputs: Vec<Vec<Vec<F>>>,
        expected_events: Vec<Vec<E>>,
        meta: M,
    ) -> Self {
        assert_eq!(expected_inputs.len(), provided_outputs.len());
        assert_eq!(expected_events.len(), expected_inputs.len());
        TestPlugin {
            expected_inputs,
            provided_outputs,
            expected_events,
            meta,
            buffer_index: 0,
            event_index: 0,
        }
    }
}

impl<F, E, M> AudioRendererMeta for TestPlugin<F, E, M>
where
    M: AudioRendererMeta,
{
    const MAX_NUMBER_OF_AUDIO_INPUTS: usize = M::MAX_NUMBER_OF_AUDIO_INPUTS;
    const MAX_NUMBER_OF_AUDIO_OUTPUTS: usize = M::MAX_NUMBER_OF_AUDIO_OUTPUTS;

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.meta.set_sample_rate(sample_rate);
    }
}

impl<F, E, M> AudioRenderer<F> for TestPlugin<F, E, M>
where
    M: AudioRendererMeta,
    F: PartialEq + Debug + Copy,
{
    fn render_buffer(&mut self, inputs: &[&[F]], outputs: &mut [&mut [F]]) {
        assert!(
            self.buffer_index < self.expected_inputs.len(),
            "`render_buffer` called more often than expected: expected only {} times",
            self.expected_inputs.len()
        );
        let expected_input_channels = &self.expected_inputs[self.buffer_index];
        assert_eq!(inputs.len(), expected_input_channels.len());
        for (input_channel_index, input_channel) in inputs.iter().enumerate() {
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

        let expected_output_channels = &self.provided_outputs[self.buffer_index];
        assert_eq!(outputs.len(), expected_output_channels.len());
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
        self.buffer_index += 1;
        self.event_index = 0;
    }
}

impl<F, E, M> EventHandler<E> for TestPlugin<F, E, M>
where
    M: AudioRendererMeta,
    E: PartialEq + Debug,
{
    fn handle_event(&mut self, event: E) {
        assert!(
            self.buffer_index < self.expected_events.len(),
            "`handle_event` is called after {} calls to `render_buffer`; this is unexpected",
            self.expected_events.len()
        );
        let expected_events_for_this_buffer = &self.expected_events[self.event_index];
        assert!(
            self.event_index < expected_events_for_this_buffer.len(),
            "`handle_events` is called more than {0} times after {1} calls to `render_buffer`;\
             only {0} times are expected because we expect only \
             {0} events for the subsequent buffer",
            expected_events_for_this_buffer.len(),
            self.buffer_index + 1
        );
        assert_eq!(
            event,
            expected_events_for_this_buffer[self.event_index],
            "mismatch for event #{} after {} calls to `render_buffer`: \
             expected {:?} but got {:?}.",
            self.event_index,
            self.buffer_index + 1,
            expected_events_for_this_buffer[self.event_index],
            event
        );
        self.event_index += 1;
    }
}
