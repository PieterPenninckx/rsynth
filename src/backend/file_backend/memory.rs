use super::AudioReader;
use crate::backend::file_backend::AudioWriter;

// TODO: Find a better name.
pub struct AudioBuffer<F> {
    // Invariant: channels is not empty.
    channels: Vec<Vec<F>>,
}

impl<F> AudioBuffer<F> {
    pub fn from_channels(channels: Vec<Vec<F>>) -> Self {
        assert!(!channels.is_empty());
        let len = channels[0].len();
        assert!(len > 0);
        for channel in channels.iter() {
            assert_eq!(len, channel.len());
        }

        Self { channels }
    }

    pub fn new(number_of_channels: usize) -> Self {
        assert!(number_of_channels > 0);
        let mut channels = Vec::with_capacity(number_of_channels);
        for _ in 0..number_of_channels {
            channels.push(Vec::new());
        }

        Self { channels }
    }

    pub fn reader<'b>(&'b self, frames_per_second: u64) -> AudioBufferReader<'b, F> {
        assert!(frames_per_second > 0);
        AudioBufferReader {
            frames_per_second,
            frame: 0,
            buffer: self,
        }
    }

    pub fn writer<'b>(&'b mut self) -> AudioBufferWriter<'b, F> {
        AudioBufferWriter { buffer: self }
    }

    pub fn channels(&self) -> &Vec<Vec<F>> {
        &self.channels
    }
}

pub struct AudioBufferReader<'b, F> {
    frames_per_second: u64,
    frame: usize,
    buffer: &'b AudioBuffer<F>,
}

impl<'b, F> AudioReader<F> for AudioBufferReader<'b, F>
where
    F: Copy,
{
    fn number_of_channels(&self) -> usize {
        self.buffer.channels.len()
    }
    fn frames_per_second(&self) -> u64 {
        self.frames_per_second
    }

    fn fill_buffer(&mut self, output: &mut [&mut [F]]) -> usize {
        // TODO: better error handling.
        assert_eq!(output.len(), self.number_of_channels());
        // Note: `self.number_of_channels() > 0`
        let buffer_size = output[0].len();
        let remainder = self.buffer.channels[0].len() - self.frame;
        let frames_to_copy = std::cmp::min(buffer_size, remainder);
        for (output_channel, input_channel) in output.iter_mut().zip(self.buffer.channels.iter()) {
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
        use super::super::{AudioBuffer, AudioBufferReader};
        use crate::dev_utilities::{create_buffers, slicify_mut};

        #[test]
        fn works_as_expected() {
            let data = vec![
                vec![1, 2, 3, 4, 5],
                vec![6, 7, 8, 9, 10],
                vec![11, 12, 13, 14, 15],
            ];
            let audio_buffer = AudioBuffer::from_channels(data);
            let mut reader = audio_buffer.reader(16);
            let mut output_buffer = create_buffers(3, 2);
            let mut buffers = slicify_mut(&mut output_buffer);
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
    buffer: &'b mut AudioBuffer<F>,
}

impl<'b, F> AudioWriter<F> for AudioBufferWriter<'b, F>
where
    F: Copy,
{
    fn write_buffer(&mut self, buffer: &[&[F]]) {
        assert_eq!(self.buffer.channels.len(), buffer.len());
        assert!(buffer.len() > 0);
        let len = buffer[0].len();
        for channel in buffer.iter() {
            assert_eq!(len, channel.len());
        }
        for (output_channel, input_channel) in self.buffer.channels.iter_mut().zip(buffer.iter()) {
            output_channel.extend_from_slice(input_channel);
        }
    }
}

#[cfg(test)]
mod AudioBufferWriterTests {
    mod write_buffer {
        use super::super::super::AudioWriter;
        use super::super::{AudioBuffer, AudioBufferWriter};
        use crate::dev_utilities::slicify;

        #[test]
        fn works_as_expected() {
            let mut audio_buffer = AudioBuffer::new(3);
            {
                let mut writer = audio_buffer.writer();

                let input = vec![vec![1, 2], vec![3, 4], vec![5, 6]];
                writer.write_buffer(slicify(&input).as_ref());
            }
            assert_eq!(audio_buffer.channels[0], vec![1, 2]);
            assert_eq!(audio_buffer.channels[1], vec![3, 4]);
            assert_eq!(audio_buffer.channels[2], vec![5, 6]);
        }
    }
}
