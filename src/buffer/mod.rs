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

/// Audio input buffer
///
/// It is guaranteed that all channels have the same length (number of samples).
#[derive(Clone, Copy)]
pub struct AudioBufferIn<'channels, 'samples, S>
where
    S: 'static + Copy,
{
    channels: &'channels [&'samples [S]],
    length: usize,
}

impl<'channels, 'samples, S> AudioBufferIn<'channels, 'samples, S>
where
    S: 'static + Copy,
{
    /// Create a new audio input buffer.
    ///
    /// # Panics
    /// Panics if one of the elements of `inputs` does not have the given length.
    pub fn new(inputs: &'channels [&'samples [S]], length: usize) -> Self {
        for channel in inputs {
            assert_eq!(channel.len(), length);
        }
        Self {
            channels: inputs,
            length,
        }
    }

    /// Get the number of channels.
    pub fn number_of_channels(&self) -> usize {
        self.channels.len()
    }

    /// Get the number of frames.
    pub fn number_of_frames(&self) -> usize {
        self.length
    }

    /// Get the channels as slices.
    pub fn channels(&self) -> &'channels [&'samples [S]] {
        self.channels
    }

    /// Get a sub-chunk with the given range of frames.
    ///
    /// The vector `vec` will be used to store the channels of the result.
    ///
    /// # Example
    /// ```
    /// use rsynth::buffer::AudioBufferIn;
    /// let mut vec = Vec::with_capacity(2);
    /// let channel1 = vec![11, 12, 13, 14];
    /// let channel2 = vec![21, 22, 23, 24];
    /// let chunk = [channel1.as_slice(), channel2.as_slice()];
    /// let chunk = AudioBufferIn::new(&chunk, 4);
    /// let parts = chunk.index_frames(1..2, &mut vec);
    /// assert_eq!(parts.number_of_frames(), 1);
    /// assert_eq!(parts.number_of_channels(), 2);
    /// let channels = parts.channels();
    /// assert_eq!(channels[0], &[12]);
    /// assert_eq!(channels[1], &[22]);
    /// ```
    ///
    /// # Remark
    /// The vector `vec` will be cleared before use in order to guarantee that all channels
    /// have the same length.
    ///
    /// # Usage in a real-time thread
    /// This method will append `number_of_channels` elements to the given vector.
    /// This will cause memory to be allocated if this exceeds the capacity of the
    /// given vector.
    ///
    /// # Suggestion
    /// You can use the [`vecstorage`] crate to re-use the memory of a vector for
    /// different lifetimes.
    ///
    /// [`vecstorage`]: https://crates.io/crates/vecstorage
    pub fn index_frames<'s, 'v, R>(
        &'s self,
        range: R,
        vec: &'v mut Vec<&'s [S]>,
    ) -> AudioBufferIn<'v, 's, S>
    where
        R: SliceIndex<[S], Output = [S]> + RangeBounds<usize> + Clone,
    {
        let length = number_of_frames_in_range(self.length, range.clone());
        let mut remaining_chunk = &*self.channels;
        vec.clear();
        while let Some((first_channel, remaining_channels)) = remaining_chunk.split_first() {
            vec.push(first_channel.index(range.clone()));
            remaining_chunk = remaining_channels;
        }
        AudioBufferIn {
            channels: vec.as_slice(),
            length,
        }
    }

    /// Get the channel with the given index.
    // TODO: maybe find a better name for this method.
    pub fn get_channel(&self, index: usize) -> Option<&[S]> {
        if index > self.channels.len() {
            None
        } else {
            Some(self.channels[index])
        }
    }

    /// Get the channel with the given index.
    ///
    /// # Panics
    /// Panics if the index is out of bounds.
    pub fn index_channel(&self, index: usize) -> &[S] {
        self.channels[index]
    }
}

#[test]
fn buffer_in_index_frames_works() {
    let mut vec = Vec::with_capacity(2);
    let channel1 = vec![11, 12, 13, 14];
    let channel2 = vec![21, 22, 23, 24];
    let chunk = [channel1.as_slice(), channel2.as_slice()];
    let chunk = AudioBufferIn::new(&chunk, 4);
    {
        let sub_part = chunk.index_frames(0..0, &mut vec);
        assert_eq!(sub_part.number_of_frames(), 0);
        assert_eq!(sub_part.number_of_channels(), 2);
        let channels = sub_part.channels();
        assert!(channels[0].is_empty());
        assert!(channels[1].is_empty());
    }
    {
        let sub_part = chunk.index_frames(0..1, &mut vec);
        assert_eq!(sub_part.number_of_frames(), 1);
        assert_eq!(sub_part.number_of_channels(), 2);
        let channels = sub_part.channels();
        assert_eq!(channels[0], &[11]);
        assert_eq!(channels[1], &[21]);
    }
    {
        let sub_part = chunk.index_frames(0..2, &mut vec);
        assert_eq!(sub_part.number_of_frames(), 2);
        assert_eq!(sub_part.number_of_channels(), 2);
        let channels = sub_part.channels();
        assert_eq!(channels[0], &[11, 12]);
        assert_eq!(channels[1], &[21, 22]);
    }
    {
        let sub_part = chunk.index_frames(1..2, &mut vec);
        assert_eq!(sub_part.number_of_frames(), 1);
        assert_eq!(sub_part.number_of_channels(), 2);
        let channels = sub_part.channels();
        assert_eq!(channels[0], &[12]);
        assert_eq!(channels[1], &[22]);
    }
}

/// An audio output buffer.
///
/// It is guaranteed that all channels have the same length (number of samples).
pub struct AudioBufferOut<'channels, 'out_samples, S>
where
    S: 'static + Copy,
{
    channels: &'channels mut [&'out_samples mut [S]],
    length: usize,
}

impl<'channels, 'samples, S> AudioBufferOut<'channels, 'samples, S>
where
    S: 'static + Copy,
{
    /// Create a new audio output buffer.
    ///
    /// # Panics
    /// Panics if one of the elements of `outputs` does not have the given length.
    pub fn new(outputs: &'channels mut [&'samples mut [S]], length: usize) -> Self {
        for channel in outputs.iter() {
            assert_eq!(channel.len(), length);
        }
        Self {
            channels: outputs,
            length,
        }
    }

    /// Get the number of channels.
    pub fn number_of_channels(&self) -> usize {
        self.channels.len()
    }

    /// Get the number of frames.
    pub fn number_of_frames(&self) -> usize {
        self.length
    }

    /// Get the channels as slices.
    ///
    /// # Unsafe
    /// This method is marked unsafe because using it allows to change the length of the
    /// channels, which invalidates the invariant
    pub unsafe fn channels<'a>(&'a mut self) -> &'a mut [&'samples mut [S]] {
        self.channels
    }

    /// Split into two `AudioBufferOut`s.
    /// The first will contain the first `mid-1` channels and the second
    /// will contain the remaining channels.
    ///
    /// # Panics
    /// Panics if `mid` is `>` the number of output channels.
    ///
    /// # Example
    /// ```
    /// use rsynth::buffer::AudioBufferOut;
    ///
    /// let mut channel1 = vec![11, 12, 13, 14];
    /// let mut channel2 = vec![21, 22, 23, 24];
    /// let mut chunk = [channel1.as_mut_slice(), channel2.as_mut_slice()];
    ///
    /// let mut buffer = AudioBufferOut::new(&mut chunk, 4);
    ///
    /// let (mut firstchannels, mut secondchannels) = buffer.split_channels_at(1);
    /// assert_eq!(firstchannels.number_of_channels(), 1);
    /// assert_eq!(secondchannels.number_of_channels(), 1);
    ///
    /// assert_eq!(firstchannels.index_channel(0), &[11, 12, 13, 14]);
    /// assert_eq!(secondchannels.index_channel(0), &[21, 22, 23, 24]);
    /// ```
    pub fn split_channels_at<'a>(
        &'a mut self,
        mid: usize,
    ) -> (
        AudioBufferOut<'a, 'samples, S>,
        AudioBufferOut<'a, 'samples, S>,
    )
    where
        'a: 'channels,
    {
        let (outputs1, outputs2) = self.channels.split_at_mut(mid);
        (
            Self {
                channels: outputs1,
                length: self.length,
            },
            Self {
                channels: outputs2,
                length: self.length,
            },
        )
    }

    /// Get a sub-chunk with the given range of frames.
    ///
    /// The vector `vec` will be used to store the channels of the result.
    ///
    /// # Example
    /// ```
    /// use rsynth::buffer::AudioBufferOut;
    ///
    /// let mut channel1 = vec![11, 12, 13, 14];
    /// let mut channel2 = vec![21, 22, 23, 24];
    /// let mut channels = [channel1.as_mut_slice(), channel2.as_mut_slice()];
    /// let number_of_channels = channels.len();
    /// let mut buffer = AudioBufferOut::new(&mut channels, 4);
    /// let mut vec = Vec::with_capacity(number_of_channels);
    /// let mut sub_part = buffer.index_frames(1..2, &mut vec);
    /// assert_eq!(sub_part.number_of_frames(), 1);
    /// assert_eq!(sub_part.number_of_channels(), number_of_channels);
    /// assert_eq!(sub_part.index_channel(0), &[12]);
    /// assert_eq!(sub_part.index_channel(1), &[22]);
    /// ```
    ///
    /// # Remark
    /// The vector `vec` will be cleared before use in order to guarantee that all channels
    /// have the same length.
    ///
    /// # Usage in a real-time threat
    /// This method will append `number_of_channels` elements to the given vector.
    /// This will cause memory to be allocated if this exceeds the capacity of the
    /// given vector.
    ///
    /// # Suggestion
    /// You can use the [`vecstorage`] crate to re-use the memory of a vector for
    /// different lifetimes.
    ///
    /// [`vecstorage`]: https://crates.io/crates/vecstorage
    pub fn index_frames<'s, 'v, R>(
        &'s mut self,
        range: R,
        vec: &'v mut Vec<&'s mut [S]>,
    ) -> AudioBufferOut<'v, 's, S>
    where
        R: SliceIndex<[S], Output = [S]> + RangeBounds<usize> + Clone,
    {
        let length = number_of_frames_in_range(self.length, range.clone());
        let mut remaining_chunk = &mut *self.channels;
        vec.clear();
        while let Some((first_channel, remaining_channels)) = remaining_chunk.split_first_mut() {
            vec.push(first_channel.index_mut(range.clone()));
            remaining_chunk = remaining_channels;
        }
        AudioBufferOut {
            channels: vec.as_mut_slice(),
            length,
        }
    }

    /// Get the channel with the given index.
    // TODO: maybe find a better name for this method.
    pub fn get_channel(&mut self, index: usize) -> Option<&mut [S]> {
        if index > self.channels.len() {
            None
        } else {
            Some(self.channels[index])
        }
    }

    /// Get the channel with the given index.
    ///
    /// # Panics
    /// Panics if the index is out of bounds.
    pub fn index_channel(&mut self, index: usize) -> &mut [S] {
        self.channels[index]
    }

    /// Set all samples to the given value.
    pub fn set(&mut self, value: S) {
        for channel in self.channels.iter_mut() {
            for sample in channel.iter_mut() {
                *sample = value;
            }
        }
    }

    /// Get an iterator over the channels
    pub fn channel_iter_mut<'a>(&'a mut self) -> AudioBufferOutChannelIteratorMut<'a, 'samples, S> {
        AudioBufferOutChannelIteratorMut {
            inner: self.channels.iter_mut(),
        }
    }

    pub fn as_audio_buffer_in<'s, 'vec>(
        &'s self,
        vec: &'vec mut Vec<&'s [S]>,
    ) -> AudioBufferIn<'vec, 's, S> {
        vec.clear();
        for channel in self.channels.iter() {
            vec.push(&**channel);
        }
        AudioBufferIn {
            channels: vec,
            length: self.length,
        }
    }
}

pub struct AudioBufferOutChannelIteratorMut<'channels, 'samples, S> {
    inner: std::slice::IterMut<'channels, &'samples mut [S]>,
}

impl<'channels, 'samples, S> Iterator for AudioBufferOutChannelIteratorMut<'channels, 'samples, S> {
    type Item = &'channels mut [S];

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|x| &mut **x)
    }
}

#[test]
fn buffer_out_index_frames_works() {
    let mut channel1 = vec![11, 12, 13, 14];
    let mut channel2 = vec![21, 22, 23, 24];
    let mut chunk = [channel1.as_mut_slice(), channel2.as_mut_slice()];
    let mut chunk = AudioBufferOut::new(&mut chunk, 4);
    {
        let mut vec = Vec::with_capacity(2);
        let mut sub_part = chunk.index_frames(0..0, &mut vec);
        assert_eq!(sub_part.number_of_frames(), 0);
        assert_eq!(sub_part.number_of_channels(), 2);
        assert!(sub_part.index_channel(0).is_empty());
        assert!(sub_part.index_channel(1).is_empty());
    }
    {
        let mut vec = Vec::with_capacity(2);
        let mut sub_part = chunk.index_frames(0..1, &mut vec);
        assert_eq!(sub_part.number_of_frames(), 1);
        assert_eq!(sub_part.number_of_channels(), 2);
        assert_eq!(sub_part.index_channel(0), &[11]);
        assert_eq!(sub_part.index_channel(1), &[21]);
    }
    {
        let mut vec = Vec::with_capacity(2);
        let mut sub_part = chunk.index_frames(0..2, &mut vec);
        assert_eq!(sub_part.number_of_frames(), 2);
        assert_eq!(sub_part.number_of_channels(), 2);
        assert_eq!(sub_part.index_channel(0), &[11, 12]);
        assert_eq!(sub_part.index_channel(1), &[21, 22]);
    }
    {
        let mut vec = Vec::with_capacity(2);
        let mut sub_part = chunk.index_frames(1..2, &mut vec);
        assert_eq!(sub_part.number_of_frames(), 1);
        assert_eq!(sub_part.number_of_channels(), 2);
        assert_eq!(sub_part.index_channel(0), &[12]);
        assert_eq!(sub_part.index_channel(1), &[22]);
    }
}

/// A buffer holding both input and output audio.
///
/// All inputs and all outputs are guaranteed to have the same number of frames.
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
    /// # Panics
    /// Panics if one of the following happens:
    /// * not all elements of `inputs` have the same length,
    /// * not all elements of `outputs` have the same length,
    /// * not all elements of `inputs` have the same length as all the elements of `outputs`
    pub fn new(
        inputs: &'in_channels [&'in_samples [S]],
        outputs: &'out_channels mut [&'out_samples mut [S]],
        length: usize,
    ) -> Self {
        let inputs = AudioBufferIn::new(inputs, length);
        let outputs = AudioBufferOut::new(outputs, length);
        assert_eq!(inputs.number_of_frames(), outputs.number_of_frames());
        AudioBufferInOut {
            inputs,
            outputs,
            length,
        }
    }

    /// Get the number of frames.
    pub fn number_of_frames(&self) -> usize {
        self.length
    }

    /// Get the number of input channels.
    pub fn number_of_input_channels(&self) -> usize {
        self.inputs.number_of_channels()
    }

    /// Get the number of output channels.
    pub fn number_of_output_channels(&self) -> usize {
        self.outputs.number_of_channels()
    }

    /// Create two new `AUdioBufferInOut`s: one with all the input channels and with the
    /// output channels from 0 to `mid`, excluding `mid` and one with all the input channels
    /// and with the output channels from `mid` including onwards.
    ///
    /// # Panics
    /// Panics if `mid` is `>` the number of output channels.
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

    /// Get the channel with the given index.
    // TODO: maybe find a better name for this method.
    pub fn get_input_channel(&self, index: usize) -> Option<&[S]> {
        self.inputs.get_channel(index)
    }

    /// Get the output channel with the given index.
    ///
    /// # Panics
    /// Panics if the index is out of bounds.
    pub fn index_input_channel(&mut self, index: usize) -> &[S] {
        self.inputs.index_channel(index)
    }

    /// Get the channel with the given index.
    // TODO: maybe find a better name for this method.
    pub fn get_output_channel(&mut self, index: usize) -> Option<&mut [S]> {
        self.outputs.get_channel(index)
    }

    /// Get the output channel with the given index.
    ///
    /// # Panics
    /// Panics if the index is out of bounds.
    pub fn index_output_channel(&mut self, index: usize) -> &mut [S] {
        self.outputs.index_channel(index)
    }

    /// Get a sub-chunk with the given range of frames.
    ///
    /// The vectors `vec_in` and `vec_out` will be used to store the channels of the result.
    ///
    /// # Example
    /// ```
    /// use rsynth::buffer::AudioBufferInOut;
    ///
    /// let channel1_in = vec![11, 12, 13, 14];
    /// let channel2_in = vec![21, 22, 23, 24];
    /// let channels_in = [channel1_in.as_slice(), channel2_in.as_slice()];
    /// let number_of_input_channels = channels_in.len();
    /// let mut channel1_out = vec![110, 120, 130, 140];
    /// let mut channels_out = [channel1_out.as_mut_slice()];
    /// let number_of_output_channels = channels_out.len();
    /// let mut buffer = AudioBufferInOut::new(&channels_in, &mut channels_out, 4);
    /// let mut vec_in = Vec::with_capacity(number_of_input_channels);
    /// let mut vec_out = Vec::with_capacity(number_of_output_channels);
    /// let mut sub_part = buffer.index_frames(1..2, &mut vec_in, &mut vec_out);
    /// assert_eq!(sub_part.number_of_frames(), 1);
    /// assert_eq!(sub_part.number_of_input_channels(), number_of_input_channels);
    /// assert_eq!(sub_part.number_of_output_channels(), number_of_output_channels);
    /// assert_eq!(sub_part.index_input_channel(0), &[12]);
    /// assert_eq!(sub_part.index_input_channel(1), &[22]);
    /// assert_eq!(sub_part.index_output_channel(0), &[120]);
    /// ```
    ///
    /// # Remark
    /// The vectors `vec_in` and `vec_out` will be cleared before use in order to guarantee that all
    /// channels have the same length.
    ///
    /// # Usage in a real-time thread
    /// This method will append `number_of_input_channels` elements to the given "input" vector
    /// and `number_of_output_channels` to the "output" vector.
    /// This will cause memory to be allocated if this exceeds the capacity of the
    /// given vector.
    ///
    /// # Suggestion
    /// You can use the [`vecstorage`] crate to re-use the memory of a vector for
    /// different lifetimes.
    ///
    /// [`vecstorage`]: https://crates.io/crates/vecstorage
    pub fn index_frames<'s, 'in_vec, 'out_vec, R>(
        &'s mut self,
        range: R,
        vec_in: &'in_vec mut Vec<&'s [S]>,
        vec_out: &'out_vec mut Vec<&'s mut [S]>,
    ) -> AudioBufferInOut<'in_vec, 's, 'out_vec, 's, S>
    where
        R: SliceIndex<[S], Output = [S]> + RangeBounds<usize> + Clone,
    {
        AudioBufferInOut {
            inputs: self.inputs.index_frames(range.clone(), vec_in),
            outputs: self.outputs.index_frames(range.clone(), vec_out),
            length: number_of_frames_in_range(self.length, range),
        }
    }

    /// Separate into an `AudioBufferIn` and an `AudioBufferOut`.
    ///
    /// # Example
    /// ```
    /// use rsynth::buffer::AudioBufferInOut;
    ///
    /// let channel1_in = vec![11, 12, 13, 14];
    /// let channel2_in = vec![21, 22, 23, 24];
    /// let channels_in = [channel1_in.as_slice(), channel2_in.as_slice()];
    /// let number_of_input_channels = channels_in.len();
    /// let mut channel1_out = vec![110, 120, 130, 140];
    /// let mut channels_out = [channel1_out.as_mut_slice()];
    /// let number_of_output_channels = channels_out.len();
    /// let mut buffer = AudioBufferInOut::new(&channels_in, &mut channels_out, 4);
    ///
    /// let (input_buffer, output_buffer) = buffer.separate();
    /// ```
    pub fn separate<'s>(
        &'s mut self,
    ) -> (
        AudioBufferIn<'in_channels, 'in_samples, S>,
        AudioBufferOut<'s, 'out_samples, S>,
    ) {
        (
            self.inputs,
            AudioBufferOut {
                channels: self.outputs.channels,
                length: self.outputs.length,
            },
        )
    }

    pub fn inputs(&self) -> &AudioBufferIn<'in_channels, 'in_samples, S> {
        &self.inputs
    }

    pub fn outputs(&mut self) -> &mut AudioBufferOut<'out_channels, 'out_samples, S> {
        &mut self.outputs
    }
}

// Alternative name: "packet"?
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AudioChunk<S> {
    // Invariant: channels is not empty.
    // TODO: This variant is currently not upheld and it's also not clear if we really need this.
    channels: Vec<Vec<S>>,
}

/// An iterator over the samples of an `AudioChunk`, operating frame by frame.
pub struct InterlacedSampleIterator<'a, S>
where
    S: Copy,
{
    channel_index: usize,
    frame_index: usize,
    chunk: &'a AudioChunk<S>,
}

impl<'a, S> InterlacedSampleIterator<'a, S>
where
    S: Copy,
{
    fn new(chunk: &'a AudioChunk<S>) -> Self {
        Self {
            channel_index: 0,
            frame_index: 0,
            chunk,
        }
    }
}

impl<'a, S> Iterator for InterlacedSampleIterator<'a, S>
where
    S: Copy,
{
    type Item = S;

    fn next(&mut self) -> Option<Self::Item> {
        if self.chunk.channels.is_empty() {
            return None;
        }
        let result = self.chunk.channels[self.channel_index]
            .get(self.frame_index)
            .cloned();
        if result.is_some() {
            self.channel_index += 1;
            if self.channel_index >= self.chunk.channels.len() {
                self.channel_index = 0;
                self.frame_index += 1;
            }
        }
        result
    }
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

    /// # Panics
    /// Panics if `number_of_channels == 0`.
    /// Panics if the number of elements yielded by the iterator is not an
    /// integer multiple of `number_of_channels`.
    pub fn from_interlaced_iterator<I>(iterator: I, number_of_channels: usize) -> Self
    where
        I: Iterator<Item = S>,
        S: Copy,
    {
        let mut result = Self::new(number_of_channels);
        let mut index = 0;
        for s in iterator {
            result.channels[index % number_of_channels].push(s.clone());
            index += 1;
        }
        assert_eq!(
            index % number_of_channels,
            0,
            "Number of elements must be an integer multiple of the number of channels."
        );
        result
    }

    /// Create an interlaced iterator from an `AudioChunk`
    /// # Example
    /// ```
    /// #[macro_use]
    /// extern crate rsynth;
    /// # fn main() {
    /// let chunk = audio_chunk![[0, 1, 2], [3, 4, 5]];
    /// let iterator = chunk.interlaced();
    /// let interlaced: Vec<_> = iterator.collect();
    /// assert_eq!(interlaced, vec![0, 3, 1, 4, 2, 5]);
    /// # }
    /// ```
    pub fn interlaced<'a>(&'a self) -> InterlacedSampleIterator<'a, S>
    where
        S: Copy,
    {
        InterlacedSampleIterator::new(self)
    }

    /// # Panics
    /// Panics if `number_of_channels == 0`.
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

#[test]
fn from_interlaced_iterator_works() {
    let input = vec![1, 2, 3, 4, 5, 6];
    let chunk = AudioChunk::from_interlaced_iterator(input.iter().cloned(), 2);
    assert_eq!(chunk.channels, vec![vec![1, 3, 5], vec![2, 4, 6]]);
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
