use super::AudioReader;

pub struct AudioBuffer<F> {
    // Is not empty.
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
            buffer: self
        }
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
        for (channel_index, output_channel) in output.iter_mut().enumerate() {
            assert_eq!(buffer_size, output_channel.len());
            output_channel[0..frames_to_copy].copy_from_slice(
                &self.buffer.channels[channel_index][self.frame..self.frame + frames_to_copy],
            );
        }
        self.frame += frames_to_copy;
        return frames_to_copy;
    }
}

#[cfg(test)]
mod AudioBufferReaderTests {
    mod fill_buffer {
        #[test]
        fn works_as_expected() {
            use super::super::{AudioBuffer, AudioBufferReader};
            use super::super::super::AudioReader;
            
            let data = vec![
                vec![1, 2, 3, 4, 5],
                vec![6, 7, 8, 9, 10],
                vec![11, 12, 13, 14, 15]
            ];
            let audio_buffer = AudioBuffer::from_channels(data);
            let mut reader = audio_buffer.reader(16);
            let mut cha1 = vec![0, 0];
            let mut cha2 = vec![0, 0];
            let mut cha3 = vec![0, 0];
            let mut buffers = vec!(cha1.as_mut(), cha2.as_mut(), cha3.as_mut());
            assert_eq!(2, reader.fill_buffer(&mut buffers));
            assert_eq!(buffers[0], &vec![1, 2][..]);
            assert_eq!(buffers[1], &vec![6, 7][..]);
            assert_eq!(buffers[2], &vec![11, 12][..]);
            assert_eq!(2, reader.fill_buffer(&mut buffers));
            assert_eq!(buffers[0], &vec![3, 4][..]);
            assert_eq!(buffers[1], &vec![8, 9][..]);
            assert_eq!(buffers[2], &vec![13, 14][..]);
            assert_eq!(1, reader.fill_buffer(&mut buffers));
            assert_eq!(buffers[0], &vec![5, 4][..]);
            assert_eq!(buffers[1], &vec![10, 9][..]);
            assert_eq!(buffers[2], &vec![15, 14][..]);
        }
    }
}

