//! In-memory backend, useful for testing.
use super::{AudioReader, AudioWriter};
use crate::buffer::{AudioBufferIn, AudioBufferOut, AudioChunk};
#[cfg(feature = "backend-combined-wav")]
use dasp_sample::{FromSample, I24};
use std::borrow::Borrow;
use std::marker::PhantomData;
#[cfg(feature = "backend-combined-wav")]
use wav::{BitDepth, Header};

/// An [`AudioReader`] that reads from a given [`AudioChunk`].
/// The generic parameter type `S` represents the sample type.
///
#[cfg_attr(
    feature = "backend-combined-wav",
    doc = "\
# Example

The following example illustrates how an [`AudioChunk`] can be generated from a wav file.

_Remark_ the example assumes the `rsynth` crate is compiled with the `backend-combined-wav`.

_Remark_ the example does not use proper error handling.
```
extern crate wav;

use std::fs::File;
use std::path::Path;
use rsynth::backend::combined::memory::AudioChunkReader;

fn create_reader_from_file(filename: &str) {
    let mut file = File::open(Path::new(filename)).unwrap();
    let (header, samples) = wav::read(&mut file).unwrap();
    let reader = AudioChunkReader::<f32, _>::from((header, samples));
}
```
"
)]
///
/// [`AudioReader`]: ../trait.AudioReader.html
/// [`AudioChunk`]: ../../../buffer/struct.AudioChunk.html
pub struct AudioChunkReader<S, T>
where
    T: Borrow<AudioChunk<S>>,
    S: Copy,
{
    frames_per_second: u64,
    frame: usize,
    chunk: T,
    phantom: PhantomData<S>,
}

impl<S, T> AudioChunkReader<S, T>
where
    T: Borrow<AudioChunk<S>>,
    S: Copy,
{
    /// Construct a new `AudioBufferReader` with the given [`AudioChunk`] and
    /// sample rate in frames per second.
    ///
    /// [`AudioChunk`]: ../../../buffer/struct.AudioChunk.html
    pub fn new(buffer: T, frames_per_second: u64) -> Self {
        Self {
            chunk: buffer,
            frames_per_second,
            frame: 0,
            phantom: PhantomData::<S>,
        }
    }
}

impl<S, T> AudioReader<S> for AudioChunkReader<S, T>
where
    T: Borrow<AudioChunk<S>>,
    S: Copy,
{
    type Err = std::convert::Infallible;
    fn number_of_channels(&self) -> usize {
        self.chunk.borrow().channels().len()
    }
    fn frames_per_second(&self) -> u64 {
        self.frames_per_second
    }

    fn fill_buffer(&mut self, output: &mut AudioBufferOut<S>) -> Result<usize, Self::Err> {
        assert_eq!(output.number_of_channels(), self.number_of_channels());
        // Note: `self.number_of_channels() > 0`
        let buffer_size = output.number_of_frames();
        let remainder = self.chunk.borrow().channels()[0].len() - self.frame;
        let frames_to_copy = std::cmp::min(buffer_size, remainder);

        for (output_channel, input_channel) in output
            .channel_iter_mut()
            .zip(self.chunk.borrow().channels().iter())
        {
            assert_eq!(buffer_size, output_channel.len());
            output_channel[0..frames_to_copy]
                .copy_from_slice(&input_channel[self.frame..self.frame + frames_to_copy]);
        }
        self.frame += frames_to_copy;
        Ok(frames_to_copy)
    }
}

/// An [`AudioReader`] that reads from a given [`AudioChunk`].
/// The generic parameter type `S` represents the sample type.
///
/// [`AudioReader`]: ../trait.AudioReader.html
/// [`AudioChunk`]: ../../../buffer/struct.AudioChunk.html
pub type AudioBufferReader<'b, S> = AudioChunkReader<S, &'b AudioChunk<S>>;

#[cfg(feature = "backend-combined-wav")]
impl<S> From<(Header, BitDepth)> for AudioChunkReader<S, AudioChunk<S>>
where
    S: Copy + FromSample<u8> + FromSample<i16> + FromSample<I24>,
{
    fn from((header, samples): (Header, BitDepth)) -> Self {
        let chunk = match samples {
            BitDepth::Eight(s) => AudioChunk::from_interlaced_iterator(
                s.iter().map(|a| S::from_sample_(*a)),
                header.channel_count as usize,
            ),
            BitDepth::Sixteen(s) => AudioChunk::from_interlaced_iterator(
                s.iter().map(|a| S::from_sample_(*a)),
                header.channel_count as usize,
            ),
            BitDepth::TwentyFour(s) => AudioChunk::from_interlaced_iterator(
                s.iter().map(|a| {
                    S::from_sample_(I24::new(*a).expect("24 bits sample should be 24 bits"))
                }),
                header.channel_count as usize,
            ),
            BitDepth::Empty => AudioChunk::new(header.channel_count as usize),
        };
        Self {
            frames_per_second: header.sampling_rate as u64,
            frame: 0,
            chunk,
            phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod AudioBufferReaderTests {
    mod fill_buffer {
        use super::super::super::AudioReader;
        use super::super::AudioBufferReader;
        use crate::buffer::{AudioBufferOut, AudioChunk};

        #[test]
        fn works_as_expected() {
            let audio_buffer =
                audio_chunk![[1, 2, 3, 4, 5], [6, 7, 8, 9, 10], [11, 12, 13, 14, 15]];
            let mut reader = AudioBufferReader::new(&audio_buffer, 16);
            let mut output_buffer = AudioChunk::zero(3, 2);
            let mut slices = output_buffer.as_mut_slices();
            {
                let mut buffers = AudioBufferOut::new(&mut slices, 2);
                assert_eq!(Ok(2), reader.fill_buffer(&mut buffers));
            }
            assert_eq!(slices[0], vec![1, 2].as_slice());
            assert_eq!(slices[1], vec![6, 7].as_slice());
            assert_eq!(slices[2], vec![11, 12].as_slice());
            {
                let mut buffers = AudioBufferOut::new(&mut slices, 2);
                assert_eq!(Ok(2), reader.fill_buffer(&mut buffers));
            }
            assert_eq!(slices[0], vec![3, 4].as_slice());
            assert_eq!(slices[1], vec![8, 9].as_slice());
            assert_eq!(slices[2], vec![13, 14].as_slice());
            {
                let mut buffers = AudioBufferOut::new(&mut slices, 2);
                assert_eq!(Ok(1), reader.fill_buffer(&mut buffers));
            }
            assert_eq!(slices[0], vec![5, 4].as_slice());
            assert_eq!(slices[1], vec![10, 9].as_slice());
            assert_eq!(slices[2], vec![15, 14].as_slice());
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
    fn write_buffer(&mut self, buffer: &AudioBufferIn<S>) -> Result<(), Self::Err> {
        self.buffer.append_sliced_chunk(buffer.channels());
        Ok(())
    }
}
