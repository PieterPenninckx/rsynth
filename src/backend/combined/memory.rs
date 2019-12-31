use super::{AudioReader, AudioWriter};
use crate::buffer::AudioChunk;

/// An [`AudioReader`] that reads from a given [`AudioChunk`].
/// The generic parameter type `S` represents the sample type.
///
/// [`AudioReader`]: ../trait.AudioReader.html
/// [`AudioChunk`]: ../../../buffer/struct.AudioChunk.html
pub struct AudioBufferReader<'b, S>
where
    S: Copy,
{
    frames_per_second: u64,
    frame: usize,
    buffer: &'b AudioChunk<S>,
}

impl<'b, S> AudioBufferReader<'b, S>
where
    S: Copy,
{
    /// Construct a new `AudioBufferReader` with the given [`AudioChunk`] and
    /// sample rate in frames per second.
    ///
    /// [`AudioChunk`]: ../../../buffer/struct.AudioChunk.html
    pub fn new(buffer: &'b AudioChunk<S>, frames_per_second: u64) -> Self {
        Self {
            buffer,
            frames_per_second,
            frame: 0,
        }
    }
}

impl<'b, S> AudioReader<S> for AudioBufferReader<'b, S>
where
    S: Copy,
{
    type Err = std::convert::Infallible;
    fn number_of_channels(&self) -> usize {
        self.buffer.channels().len()
    }
    fn frames_per_second(&self) -> u64 {
        self.frames_per_second
    }

    fn fill_buffer(&mut self, output: &mut [&mut [S]]) -> Result<usize, Self::Err> {
        // TODO: better error handling.
        assert_eq!(output.len(), self.number_of_channels());
        // Note: `self.number_of_channels() > 0`
        let buffer_size = output[0].len();
        let remainder = self.buffer.channels()[0].len() - self.frame;
        let frames_to_copy = std::cmp::min(buffer_size, remainder);
        for (output_channel, input_channel) in output.iter_mut().zip(self.buffer.channels().iter())
        {
            assert_eq!(buffer_size, output_channel.len());
            output_channel[0..frames_to_copy]
                .copy_from_slice(&input_channel[self.frame..self.frame + frames_to_copy]);
        }
        self.frame += frames_to_copy;
        Ok(frames_to_copy)
    }
}

#[cfg(test)]
mod AudioBufferReaderTests {
    mod fill_buffer {
        use super::super::super::AudioReader;
        use super::super::AudioBufferReader;
        use crate::buffer::AudioChunk;

        #[test]
        fn works_as_expected() {
            let audio_buffer =
                audio_chunk![[1, 2, 3, 4, 5], [6, 7, 8, 9, 10], [11, 12, 13, 14, 15]];
            let mut reader = AudioBufferReader::new(&audio_buffer, 16);
            let mut output_buffer = AudioChunk::zero(3, 2);
            let mut buffers = output_buffer.as_mut_slices();
            assert_eq!(Ok(2), reader.fill_buffer(buffers.as_mut_slice()));
            assert_eq!(buffers[0], vec![1, 2].as_slice());
            assert_eq!(buffers[1], vec![6, 7].as_slice());
            assert_eq!(buffers[2], vec![11, 12].as_slice());
            assert_eq!(Ok(2), reader.fill_buffer(buffers.as_mut_slice()));
            assert_eq!(buffers[0], vec![3, 4].as_slice());
            assert_eq!(buffers[1], vec![8, 9].as_slice());
            assert_eq!(buffers[2], vec![13, 14].as_slice());
            assert_eq!(Ok(1), reader.fill_buffer(buffers.as_mut_slice()));
            assert_eq!(buffers[0], vec![5, 4].as_slice());
            assert_eq!(buffers[1], vec![10, 9].as_slice());
            assert_eq!(buffers[2], vec![15, 14].as_slice());
        }
    }
}

/// An [`AudioWriter`] that appends to a given [`AudioChunk`].
/// The generic parameter type `S` represents the sample type.
///
/// Note about using in a real-time context
/// =======================================
/// Because this appends to an [`AudioChunk`], it may allocate memory
/// when the capacity of the [`AudioChunk`] is exceeded.
///
/// [`AudioWriter`]: ../trait.AudioWriter.html
/// [`AudioChunk`]: ../../../buffer/struct.AudioChunk.html
pub struct AudioBufferWriter<'b, S> {
    buffer: &'b mut AudioChunk<S>,
}

impl<'b, S> AudioBufferWriter<'b, S> {
    pub fn new(buffer: &'b mut AudioChunk<S>) -> Self {
        Self { buffer }
    }
}

impl<'b, S> AudioWriter<S> for AudioBufferWriter<'b, S>
where
    S: Copy,
{
    type Err = std::convert::Infallible;
    fn write_buffer(&mut self, buffer: &[&[S]]) -> Result<(), Self::Err> {
        Ok(self.buffer.append_sliced_chunk(buffer))
    }
}
