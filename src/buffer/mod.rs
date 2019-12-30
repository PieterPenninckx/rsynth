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
use std::ops::{Deref, DerefMut};

// Alternative name: "packet"?
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AudioChunk<F> {
    // Invariant: channels is not empty.
    // TODO: This variant is currently not upheld and it's also not clear if we really need this.
    channels: Vec<Vec<F>>,
}

impl<F> AudioChunk<F> {
    // TODO: what we really want here, is to generate "silence" (equilibrium), this does not need to be equal to zero.
    /// Note: cannot be used in a real-time context
    /// -------------------------------------
    /// This method allocates memory and cannot be used in a real-time context.
    pub fn zero(number_of_channels: usize, number_of_frames: usize) -> Self
    where
        F: Zero,
    {
        let mut buffers = Vec::with_capacity(number_of_channels);
        for _ in 0..number_of_channels {
            let mut buffer = Vec::with_capacity(number_of_frames);
            for _ in 0..number_of_frames {
                buffer.push(F::zero());
            }
            buffers.push(buffer);
        }
        Self { channels: buffers }
    }

    pub fn from_channels(channels: Vec<Vec<F>>) -> Self {
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

    pub fn channels(&self) -> &Vec<Vec<F>> {
        &self.channels
    }

    /// Note about using in a real-time context
    /// ---------------------------------------
    /// This method will allocate memory if the capacity of the chunk is exceeded and cannot
    /// be used in a real-time context in that case.
    pub fn append_sliced_chunk(&mut self, chunk: &[&[F]])
    where
        F: Clone,
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

    pub fn inner(self) -> Vec<Vec<F>> {
        self.channels
    }

    /// Note: cannot be used in a real-time context
    /// -------------------------------------
    /// This method allocates memory and cannot be used in a real-time context.
    pub fn as_slices<'a>(&'a self) -> Vec<&[F]> {
        self.channels
            .iter()
            .map(|element| element.as_slice())
            .collect()
    }

    /// Note: cannot be used in a real-time context
    /// -------------------------------------
    /// This method allocates memory and cannot be used in a real-time context.
    pub fn as_mut_slices<'a>(&'a mut self) -> Vec<&mut [F]> {
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

pub fn buffers_as_slice<'a, F>(buffers: &'a Vec<Vec<F>>, slice_len: usize) -> Vec<&'a [F]> {
    buffers.iter().map(|b| &b[0..slice_len]).collect()
}

pub fn buffers_as_mut_slice<'a, F>(
    buffers: &'a mut Vec<Vec<F>>,
    slice_len: usize,
) -> Vec<&'a mut [F]> {
    buffers.iter_mut().map(|b| &mut b[0..slice_len]).collect()
}

pub struct InputChunk<'a, S> {
    number_of_frames: usize,
    channels: &'a [&'a [S]],
}

impl<'a, S> InputChunk<'a, S> {
    pub fn new(number_of_frames: usize, channels: &'a [&'a [S]]) -> Self {
        Self {
            number_of_frames,
            channels,
        }
    }

    pub fn number_of_frames(&self) -> usize {
        self.number_of_frames
    }
}

impl<'a, S> Deref for InputChunk<'a, S> {
    type Target = &'a [&'a [S]];

    fn deref(&self) -> &Self::Target {
        &self.channels
    }
}

pub struct OutputChunk<'a, 'b, S> {
    number_of_frames: usize,
    channels: &'a mut [&'b mut [S]],
}

impl<'a, 'b, S> OutputChunk<'a, 'b, S> {
    pub fn new(number_of_frames: usize, channels: &'a mut [&'b mut [S]]) -> Self {
        Self {
            number_of_frames,
            channels,
        }
    }

    pub fn number_of_frames(&self) -> usize {
        self.number_of_frames
    }
}

impl<'a, 'b, S> OutputChunk<'a, 'b, S>
where
    S: Zero,
{
    // TODO: what we really want is silence (equilibrium).
    pub fn initialize_to_zero(&mut self) {
        for channel in self.channels.iter_mut() {
            for sample in channel.iter_mut() {
                *sample = S::zero();
            }
        }
    }
}

impl<'a, 'b, S> Deref for OutputChunk<'a, 'b, S> {
    type Target = &'a mut [&'b mut [S]];

    fn deref(&self) -> &Self::Target {
        &self.channels
    }
}

impl<'a, 'b, S> DerefMut for OutputChunk<'a, 'b, S> {
    fn deref_mut(&mut self) -> &mut &'a mut [&'b mut [S]] {
        &mut self.channels
    }
}
