//! Audio buffers.
//!
//! ## Some audio concepts
//! A *sample* is a single number representing the air pressure at a given time.
//! It is usually represented by an `f32`, `f64`, `i16` or `i32` number, but other
//! types are possible as well.
//!
//! A *channel* usually corresponds with a speaker or a number of speakers.
//! E.g. in a stereo setup, there is a "left" channel and a "right" channel.
//!
//! A *frame* consists of the samples for all the channels at a given time.
//!
//! A *buffer* consists of subsequent samples for a given channel and corresponds
//! to a certain time period.
//! (Non-standard terminology.)
//!
//! A *chunk* consists of the buffers for all channels for a given time period.
//! (Non-standard terminology.)
//!
//!```text
//!                         ┌ chunk     ┌ frame
//!             ┌ sample    ↓           ↓
//!             │      ┌─────────┐     ┌─┐
//!          ┌──↓──────┼─────────┼─────┼─┼───────────────────┐
//! channel →│• • • • •│• • • • •│• • •│•│• • • • • • • • • •│
//!          └─────────┼─────────┼─────┼─┼───────────────────┘
//!           • • • • •│• • • • •│• • •│•│• • • • • • • • • •
//!                    │         │     │ │   ┌───────┐
//!           • • • • •│• • • • •│• • •│•│• •│• • • •│• • • •
//!                    └─────────┘     └─┘   └───────┘
//!                                            ↑
//!                                            └ buffer
//! ```
use num_traits::Zero;
use std::mem;
use std::ops::{Bound, Index, IndexMut, RangeBounds};
use std::slice::SliceIndex;

fn number_of_frames_in_range<R: RangeBounds<usize>>(number_of_frames: usize, range: R) -> usize {
    // start: inclusive
    let start = match range.start_bound() {
        Bound::Unbounded => 0,
        Bound::Excluded(x) => x + 1,
        Bound::Included(x) => *x,
    };
    // end: inclusive
    let end = match range.end_bound() {
        Bound::Unbounded => number_of_frames,
        Bound::Excluded(x) => *x,
        Bound::Included(x) => x + 1,
    };
    if start < end {
        end - start
    } else {
        0
    }
}

#[test]
fn number_of_frames_in_range_works_full_range() {
    assert_eq!(number_of_frames_in_range(4, ..), 4);
}

#[test]
fn number_of_frames_in_range_works_exclusive_range() {
    assert_eq!(number_of_frames_in_range(4, 1..3), 2);
}

#[test]
fn number_of_frames_in_range_works_inclusive_range() {
    assert_eq!(number_of_frames_in_range(4, 1..=3), 3);
}

#[test]
fn number_of_frames_in_range_works_open_ended_range() {
    assert_eq!(number_of_frames_in_range(4, 1..), 3);
}

#[test]
fn number_of_frames_in_range_works_open_starting_range() {
    assert_eq!(number_of_frames_in_range(4, ..2), 2);
}

#[derive(Clone, Copy)]
pub struct AudioBufferIn<'in_channels, 'in_samples, S>
where
    S: 'static + Copy,
{
    inputs: &'in_channels [&'in_samples [S]],
    length: usize,
}

impl<'in_channels, 'in_samples, S> AudioBufferIn<'in_channels, 'in_samples, S>
where
    S: 'static + Copy,
{
    /// # Panics
    /// Panics if one of the elements of `inputs` does not have the given length.
    pub fn new(inputs: &'in_channels [&'in_samples [S]], length: usize) -> Self {
        for channel in inputs {
            assert_eq!(channel.len(), length);
        }
        Self { inputs, length }
    }

    pub fn number_of_channels(&self) -> usize {
        self.inputs.len()
    }

    pub fn number_of_frames(&self) -> usize {
        self.length
    }

    pub fn channels(&self) -> &'in_channels [&'in_samples [S]] {
        self.inputs
    }

    /// # Remark
    /// The vector `vec` will be cleared before use in order to guarantee that all channels
    /// have the same length.
    pub fn index_samples<'v, R: SliceIndex<[S], Output = [S]> + RangeBounds<usize> + Clone>(
        &self,
        range: R,
        vec: &'v mut Vec<&'in_samples [S]>,
    ) -> AudioBufferIn<'in_channels, 'in_samples, S>
    where
        'v: 'in_channels,
    {
        // Clear the vector in order to guarantee that all channels have the same length.
        vec.clear();
        let mut remaining_chunk = self.inputs;
        while let Some((first_channel, remaining_channels)) = remaining_chunk.split_first() {
            vec.push(first_channel.index(range.clone()));
            remaining_chunk = remaining_channels;
        }
        Self {
            inputs: vec.as_slice(),
            length: number_of_frames_in_range(self.length, range.clone()),
        }
    }
}

#[test]
fn buffer_in_index_samples_works() {
    let mut vec = Vec::with_capacity(2);
    let channel1 = vec![11, 12, 13, 14];
    let channel2 = vec![21, 22, 23, 24];
    let chunk = [channel1.as_slice(), channel2.as_slice()];
    let chunk = AudioBufferIn::new(&chunk, 4);
    {
        let parts = chunk.index_samples(0..0, &mut vec);
        assert_eq!(parts.number_of_frames(), 0);
        assert_eq!(parts.number_of_channels(), 2);
        let channels = parts.channels();
        assert!(channels[0].is_empty());
        assert!(channels[1].is_empty());
    }
    {
        let parts = chunk.index_samples(0..1, &mut vec);
        assert_eq!(parts.number_of_frames(), 1);
        assert_eq!(parts.number_of_channels(), 2);
        let channels = parts.channels();
        assert_eq!(channels[0], &[11]);
        assert_eq!(channels[1], &[21]);
    }
    {
        let parts = chunk.index_samples(0..2, &mut vec);
        assert_eq!(parts.number_of_frames(), 2);
        assert_eq!(parts.number_of_channels(), 2);
        let channels = parts.channels();
        assert_eq!(channels[0], &[11, 12]);
        assert_eq!(channels[1], &[21, 22]);
    }
    {
        let parts = chunk.index_samples(1..2, &mut vec);
        assert_eq!(parts.number_of_frames(), 1);
        assert_eq!(parts.number_of_channels(), 2);
        let channels = parts.channels();
        assert_eq!(channels[0], &[12]);
        assert_eq!(channels[1], &[22]);
    }
}

pub struct AudioBufferOut<'out_channels, 'out_samples, S>
where
    S: 'static + Copy,
{
    outputs: &'out_channels mut [&'out_samples mut [S]],
    length: usize,
}

fn index_samples_slice<'v, 'out_channels, 'out_samples, R, S: 'static + Copy>(
    mut remaining_chunk: &'out_channels mut [&'out_samples mut [S]],
    range: R,
    vec: &'v mut Vec<&'out_channels mut [S]>,
    length: usize,
) -> AudioBufferOut<'v, 'out_channels, S>
where
    R: SliceIndex<[S], Output = [S]> + RangeBounds<usize> + Clone,
{
    vec.clear();
    while let Some((first_channel, remaining_channels)) = remaining_chunk.split_first_mut() {
        vec.push(first_channel.index_mut(range.clone()));
        remaining_chunk = remaining_channels;
    }
    AudioBufferOut {
        outputs: vec.as_mut_slice(),
        length,
    }
}

impl<'out_channels, 'out_samples, S> AudioBufferOut<'out_channels, 'out_samples, S>
where
    S: 'static + Copy,
{
    /// # Panics
    /// Panics if one of the elements of `outputs` does not have the given length.
    pub fn new(outputs: &'out_channels mut [&'out_samples mut [S]], length: usize) -> Self {
        for channel in outputs.iter() {
            assert_eq!(channel.len(), length);
        }
        Self { outputs, length }
    }

    pub fn number_of_channels(&self) -> usize {
        self.outputs.len()
    }

    pub fn number_of_frames(&self) -> usize {
        self.length
    }

    /// # Unsafe
    /// This method is marked unsafe because using it allows to change the length of the
    /// channels, which invalidates the invariant
    pub unsafe fn channels<'a>(&'a mut self) -> &'a mut [&'out_samples mut [S]] {
        self.outputs
    }

    pub fn split_channels_at<'a>(
        &'a mut self,
        mid: usize,
    ) -> (
        AudioBufferOut<'a, 'out_samples, S>,
        AudioBufferOut<'a, 'out_samples, S>,
    )
    where
        'a: 'out_channels,
    {
        let (outputs1, outputs2) = self.outputs.split_at_mut(mid);
        (
            Self {
                outputs: outputs1,
                length: self.length,
            },
            Self {
                outputs: outputs2,
                length: self.length,
            },
        )
    }

    /// # Remark
    /// The vector `vec` will be cleared before use in order to guarantee that all channels
    /// have the same length.
    pub fn index_samples<'s, 'v, R: SliceIndex<[S], Output = [S]> + RangeBounds<usize> + Clone>(
        &'s mut self,
        range: R,
        vec: &'v mut Vec<&'s mut [S]>,
    ) -> AudioBufferOut<'v, 's, S>
where {
        let length = number_of_frames_in_range(self.length, range.clone());
        index_samples_slice(self.outputs, range, vec, length)
    }

    // TODO: maybe find a better name for this method.
    pub fn get_channel(&mut self, index: usize) -> Option<&mut [S]> {
        if index > self.outputs.len() {
            None
        } else {
            Some(self.outputs[index])
        }
    }

    pub fn index_channel(&mut self, index: usize) -> &mut [S] {
        self.outputs[index]
    }
}

#[test]
fn buffer_out_index_samples_works() {
    let mut channel1 = vec![11, 12, 13, 14];
    let mut channel2 = vec![21, 22, 23, 24];
    let mut chunk = [channel1.as_mut_slice(), channel2.as_mut_slice()];
    let mut chunk = AudioBufferOut::new(&mut chunk, 4);
    {
        let mut vec = Vec::with_capacity(2);
        let mut parts = chunk.index_samples(0..0, &mut vec);
        assert_eq!(parts.number_of_frames(), 0);
        assert_eq!(parts.number_of_channels(), 2);
        assert!(parts.index_channel(0).is_empty());
        assert!(parts.index_channel(1).is_empty());
    }
    {
        let mut vec = Vec::with_capacity(2);
        let mut parts = chunk.index_samples(0..1, &mut vec);
        assert_eq!(parts.number_of_frames(), 1);
        assert_eq!(parts.number_of_channels(), 2);
        assert_eq!(parts.index_channel(0), &[11]);
        assert_eq!(parts.index_channel(1), &[21]);
    }
    {
        let mut vec = Vec::with_capacity(2);
        let mut parts = chunk.index_samples(0..2, &mut vec);
        assert_eq!(parts.number_of_frames(), 2);
        assert_eq!(parts.number_of_channels(), 2);
        assert_eq!(parts.index_channel(0), &[11, 12]);
        assert_eq!(parts.index_channel(1), &[21, 22]);
    }
    {
        let mut vec = Vec::with_capacity(2);
        let mut parts = chunk.index_samples(1..2, &mut vec);
        assert_eq!(parts.number_of_frames(), 1);
        assert_eq!(parts.number_of_channels(), 2);
        assert_eq!(parts.index_channel(0), &[12]);
        assert_eq!(parts.index_channel(1), &[22]);
    }
}

pub struct AudioBufferInOut<'in_channels, 'in_samples, 'out_channels, 'out_samples, S>
where
    S: 'static + Copy,
{
    inputs: AudioBufferIn<'in_channels, 'in_samples, S>,
    outputs: AudioBufferOut<'out_channels, 'out_samples, S>,
    length: usize,
}

impl<'in_channels, 'in_samples, 'out_channels, 'out_samples, S>
    AudioBufferInOut<'in_channels, 'in_samples, 'out_channels, 'out_samples, S>
where
    S: 'static + Copy,
{
    pub fn new(
        inputs: &'in_channels [&'in_samples [S]],
        outputs: &'out_channels mut [&'out_samples mut [S]],
        length: usize,
    ) -> Self {
        AudioBufferInOut {
            inputs: AudioBufferIn::new(inputs, length),
            outputs: AudioBufferOut::new(outputs, length),
            length,
        }
    }

    pub fn split_output_channels_at<'a>(
        &'a mut self,
        mid: usize,
    ) -> (
        AudioBufferInOut<'in_channels, 'in_samples, 'a, 'out_samples, S>,
        AudioBufferInOut<'in_channels, 'in_samples, 'a, 'out_samples, S>,
    )
    where
        'a: 'out_channels,
    {
        let (outputs1, outputs2) = self.outputs.split_channels_at(mid);
        (
            Self {
                inputs: self.inputs,
                outputs: outputs1,
                length: self.length,
            },
            Self {
                inputs: self.inputs,
                outputs: outputs2,
                length: self.length,
            },
        )
    }
}

// Alternative name: "packet"?
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AudioChunk<S> {
    // Invariant: channels is not empty.
    // TODO: This variant is currently not upheld and it's also not clear if we really need this.
    channels: Vec<Vec<S>>,
}

impl<S> AudioChunk<S> {
    // TODO: what we really want here, is to generate "silence" (equilibrium), this does not need to be equal to zero.
    /// Note: cannot be used in a real-time context
    /// -------------------------------------
    /// This method allocates memory and cannot be used in a real-time context.
    pub fn zero(number_of_channels: usize, number_of_frames: usize) -> Self
    where
        S: Zero,
    {
        let mut buffers = Vec::with_capacity(number_of_channels);
        for _ in 0..number_of_channels {
            let mut buffer = Vec::with_capacity(number_of_frames);
            for _ in 0..number_of_frames {
                buffer.push(S::zero());
            }
            buffers.push(buffer);
        }
        Self { channels: buffers }
    }

    pub fn from_channels(channels: Vec<Vec<S>>) -> Self {
        assert!(!channels.is_empty());
        let len = channels[0].len();
        assert!(len > 0);
        for channel in channels.iter() {
            assert_eq!(len, channel.len());
        }

        Self { channels }
    }

    /// Note: cannot be used in a real-time context
    /// -------------------------------------
    /// This method allocates memory and cannot be used in a real-time context.
    pub fn new(number_of_channels: usize) -> Self {
        assert!(number_of_channels > 0);
        let mut channels = Vec::with_capacity(number_of_channels);
        for _ in 0..number_of_channels {
            channels.push(Vec::new());
        }

        Self { channels }
    }

    /// Create a new `AudioChunk` in which each channel has the given capacity.
    /// This allows to append `capacity` frames to the `AudioChunk` (e.g. by calling
    /// `append_sliced_chunk`).
    ///
    /// Note: cannot be used in a real-time context
    /// -------------------------------------
    /// This method allocates memory and cannot be used in a real-time context.
    pub fn with_capacity(number_of_channels: usize, capacity: usize) -> Self {
        assert!(number_of_channels > 0);
        let mut channels = Vec::with_capacity(number_of_channels);
        for _ in 0..number_of_channels {
            channels.push(Vec::with_capacity(capacity));
        }

        Self { channels }
    }

    pub fn channels(&self) -> &Vec<Vec<S>> {
        &self.channels
    }

    /// Note about using in a real-time context
    /// ---------------------------------------
    /// This method will allocate memory if the capacity of the chunk is exceeded and cannot
    /// be used in a real-time context in that case.
    pub fn append_sliced_chunk(&mut self, chunk: &[&[S]])
    where
        S: Clone,
    {
        assert_eq!(self.channels.len(), chunk.len());
        let len = chunk[0].len();
        for channel in chunk.iter() {
            assert_eq!(len, channel.len());
        }
        for (output_channel, input_channel) in self.channels.iter_mut().zip(chunk.iter()) {
            output_channel.extend_from_slice(input_channel);
        }
    }

    pub fn inner(self) -> Vec<Vec<S>> {
        self.channels
    }

    /// Note: cannot be used in a real-time context
    /// -------------------------------------
    /// This method allocates memory and cannot be used in a real-time context.
    pub fn as_slices<'a>(&'a self) -> Vec<&[S]> {
        self.channels
            .iter()
            .map(|element| element.as_slice())
            .collect()
    }

    /// Note: cannot be used in a real-time context
    /// -------------------------------------
    /// This method allocates memory and cannot be used in a real-time context.
    pub fn as_mut_slices<'a>(&'a mut self) -> Vec<&mut [S]> {
        self.channels
            .iter_mut()
            .map(|element| element.as_mut_slice())
            .collect()
    }

    /// Note: cannot be used in a real-time context
    /// -------------------------------------
    /// This method allocates memory and cannot be used in a real-time context.
    pub fn split(mut self, number_of_frames_per_chunk: usize) -> Vec<Self> {
        assert!(number_of_frames_per_chunk > 0);

        let number_of_samples = self.channels[0].len();

        let result_len = number_of_samples / number_of_frames_per_chunk
            + if number_of_samples % number_of_frames_per_chunk == 0 {
                0
            } else {
                1
            };

        let mut result = Vec::with_capacity(result_len);
        for _ in 0..result_len {
            result.push(Vec::new());
        }

        for mut channel in self.channels.drain(..) {
            let mut chunk_index = 0;
            let mut chunk = Vec::new();
            for sample in channel.drain(..) {
                chunk.push(sample);
                if chunk.len() == number_of_frames_per_chunk {
                    result[chunk_index].push(mem::replace(&mut chunk, Vec::new()));
                    chunk_index += 1;
                }
            }
            if !chunk.is_empty() {
                result[chunk_index].push(chunk);
            }
        }
        result.drain(..).map(AudioChunk::from_channels).collect()
    }
}

#[macro_export]
/// Create an audio chunk.
/// ## Example
/// ```
/// // Create an audio chunk with two channels and three frames.
/// # #[macro_use]
/// # extern crate rsynth;
/// # fn main() {
/// let input = audio_chunk![[1, 2], [3, 4], [5, 6]];
/// # }
/// ```
macro_rules! audio_chunk {
    [
        [
            $head_head:expr
            $(
                , $head_tail: expr
            )*
        ]
        $(
            ,
            [
                $tail_head:expr
                $(
                    , $tail_tail: expr
                )*
            ]
        )*
    ] => {
        $crate::buffer::AudioChunk::from_channels(
            vec![
                vec![
                    $head_head
                    $(
                        , $head_tail
                    )*
                ]
                $(
                    , vec![
                        $tail_head
                        $(
                            , $tail_tail
                        )*
                    ]
                )*
            ]
        )
    };
}

#[test]
fn append_works_as_expected() {
    let mut audio_buffer = AudioChunk::new(3);
    let input = audio_chunk![[1, 2], [3, 4], [5, 6]];
    audio_buffer.append_sliced_chunk(input.as_slices().as_ref());
    assert_eq!(audio_buffer.channels()[0], vec![1, 2]);
    assert_eq!(audio_buffer.channels()[1], vec![3, 4]);
    assert_eq!(audio_buffer.channels()[2], vec![5, 6]);
}

#[test]
fn split_works_with_dividing_input_length() {
    let input = audio_chunk![[0, 1, 2, 3], [5, 6, 7, 8]];
    let observed = input.split(2);
    assert_eq!(
        observed,
        vec![audio_chunk![[0, 1], [5, 6]], audio_chunk![[2, 3], [7, 8]]]
    )
}

#[test]
fn split_works_with_non_dividing_input_length() {
    let input = audio_chunk![[0, 1, 2, 3, 4], [5, 6, 7, 8, 9]];
    let observed = input.split(2);
    assert_eq!(
        observed,
        vec![
            audio_chunk![[0, 1], [5, 6]],
            audio_chunk![[2, 3], [7, 8]],
            audio_chunk![[4], [9]]
        ]
    )
}

pub fn buffers_as_slice<'a, S>(buffers: &'a [Vec<S>], slice_len: usize) -> Vec<&'a [S]> {
    buffers.iter().map(|b| &b[0..slice_len]).collect()
}

pub fn buffers_as_mut_slice<'a, S>(
    buffers: &'a mut [Vec<S>],
    slice_len: usize,
) -> Vec<&'a mut [S]> {
    buffers.iter_mut().map(|b| &mut b[0..slice_len]).collect()
}

/// Initialize a slice of buffers to zero.
// TODO: what we really want is silence (equilibrium).
pub fn initialize_to_zero<S: num_traits::Zero>(buffers: &mut [&mut [S]]) {
    for buffer in buffers.iter_mut() {
        for sample in buffer.iter_mut() {
            *sample = S::zero();
        }
    }
}
