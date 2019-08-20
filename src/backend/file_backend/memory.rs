use super::AudioReader;
use crate::backend::file_backend::AudioWriter;
use crate::dev_utilities::chunk::AudioChunk;

pub struct AudioBufferReader<'b, F> {
    frames_per_second: u64,
    frame: usize,
    buffer: &'b AudioChunk<F>,
}

impl<'b, F> AudioBufferReader<'b, F> {
    pub fn new(buffer: &'b AudioChunk<F>, frames_per_second: u64) -> Self {
        Self {
            buffer,
            frames_per_second,
            frame: 0,
        }
    }
}

impl<'b, F> AudioReader<F> for AudioBufferReader<'b, F>
where
    F: Copy,
{
    fn number_of_channels(&self) -> usize {
        self.buffer.channels().len()
    }
    fn frames_per_second(&self) -> u64 {
        self.frames_per_second
    }

    fn fill_buffer(&mut self, output: &mut [&mut [F]]) -> usize {
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
        return frames_to_copy;
    }
}

#[cfg(test)]
mod AudioBufferReaderTests {
    mod fill_buffer {
        use super::super::super::AudioReader;
        use super::super::AudioBufferReader;
        use crate::dev_utilities::chunk::AudioChunk;

        #[test]
        fn works_as_expected() {
            let audio_buffer =
                audio_chunk![[1, 2, 3, 4, 5], [6, 7, 8, 9, 10], [11, 12, 13, 14, 15]];
            let mut reader = AudioBufferReader::new(&audio_buffer, 16);
            let mut output_buffer = AudioChunk::zero(3, 2);
            let mut buffers = output_buffer.as_mut_slices();
            assert_eq!(2, reader.fill_buffer(buffers.as_mut_slice()));
            assert_eq!(buffers[0], vec![1, 2].as_slice());
            assert_eq!(buffers[1], vec![6, 7].as_slice());
            assert_eq!(buffers[2], vec![11, 12].as_slice());
            assert_eq!(2, reader.fill_buffer(buffers.as_mut_slice()));
            assert_eq!(buffers[0], vec![3, 4].as_slice());
            assert_eq!(buffers[1], vec![8, 9].as_slice());
            assert_eq!(buffers[2], vec![13, 14].as_slice());
            assert_eq!(1, reader.fill_buffer(buffers.as_mut_slice()));
            assert_eq!(buffers[0], vec![5, 4].as_slice());
            assert_eq!(buffers[1], vec![10, 9].as_slice());
            assert_eq!(buffers[2], vec![15, 14].as_slice());
        }
    }
}

pub struct AudioBufferWriter<'b, F> {
    buffer: &'b mut AudioChunk<F>,
}

impl<'b, F> AudioBufferWriter<'b, F> {
    pub fn new(buffer: &'b mut AudioChunk<F>) -> Self {
        Self { buffer }
    }
}

impl<'b, F> AudioWriter<F> for AudioBufferWriter<'b, F>
where
    F: Copy,
{
    fn write_buffer(&mut self, buffer: &[&[F]]) {
        self.buffer.append_sliced_chunk(buffer);
    }
}
