use super::{AudioReader, AudioWriter};
use crate::buffer::{AudioBufferIn, AudioBufferOut};
use hound::{WavReader, WavSamples, WavWriter};
use sample::conv::{FromSample, ToSample};
use std::io::{Read, Seek, Write};

pub struct HoundAudioReader<'wr, S>
where
    S: FromSample<f32> + FromSample<i32> + FromSample<i16>,
{
    hound_sample_reader: Box<dyn HoundSampleReader<S> + 'wr>,
    number_of_channels: usize,
    frames_per_second: u64,
}

pub enum HoundAudioError {
    UnsupportedAudioFormat,
}

impl<'wr, S> HoundAudioReader<'wr, S>
where
    S: FromSample<f32> + FromSample<i32> + FromSample<i16>,
{
    fn reader<R: Read>(
        r: &'wr mut WavReader<R>,
    ) -> Result<Box<dyn HoundSampleReader<S> + 'wr>, HoundAudioError> {
        let spec = r.spec();
        Ok(match spec.sample_format {
            hound::SampleFormat::Float => match spec.bits_per_sample {
                32 => Box::new(F32SampleReader {
                    samples: r.samples(),
                }),
                _ => {
                    return Err(HoundAudioError::UnsupportedAudioFormat);
                }
            },
            hound::SampleFormat::Int => match spec.bits_per_sample {
                24 | 32 => Box::new(I32SampleReader {
                    samples: r.samples(),
                }),
                8 | 16 => Box::new(I16SampleReader {
                    samples: r.samples(),
                }),
                _ => {
                    // Note: until 3.4.0, Hound only supports 8, 16, 24, 32 bits/sample.
                    // Something else (e.g. 12 bits) would result in an error at runtime,
                    // so it does not make sense to allow this at this point.
                    return Err(HoundAudioError::UnsupportedAudioFormat);
                }
            },
        })
    }

    pub fn new<R: Read>(reader: &'wr mut WavReader<R>) -> Result<Self, HoundAudioError> {
        let spec = reader.spec();

        let number_of_channels = spec.channels as usize;
        let hound_sample_reader = Self::reader(reader)?;
        Ok(Self {
            number_of_channels,
            frames_per_second: spec.sample_rate as u64,
            hound_sample_reader,
        })
    }
}

impl<'wr, S> AudioReader<S> for HoundAudioReader<'wr, S>
where
    S: Copy + FromSample<f32> + FromSample<i32> + FromSample<i16>,
{
    type Err = hound::Error;

    fn number_of_channels(&self) -> usize {
        self.number_of_channels
    }

    fn frames_per_second(&self) -> u64 {
        self.frames_per_second
    }

    fn fill_buffer(&mut self, outputs: &mut AudioBufferOut<S>) -> Result<usize, Self::Err> {
        assert_eq!(outputs.number_of_channels(), self.number_of_channels());
        let length = outputs.number_of_frames();
        let mut frame_index = 0;
        while frame_index < length {
            for output in outputs.iter_channel_mut() {
                if let Some(sample) = self.hound_sample_reader.read_sample()? {
                    output[frame_index] = sample;
                } else {
                    return Ok(frame_index);
                }
            }
            frame_index += 1;
        }
        Ok(frame_index)
    }
}

trait HoundSampleReader<S> {
    fn read_sample(&mut self) -> Result<Option<S>, hound::Error>;
}

struct F32SampleReader<'wr, R: Read> {
    samples: WavSamples<'wr, R, f32>,
}

impl<'wr, R: Read, S> HoundSampleReader<S> for F32SampleReader<'wr, R>
where
    S: FromSample<f32>,
{
    fn read_sample(&mut self) -> Result<Option<S>, hound::Error> {
        if let Some(n) = self.samples.next() {
            Ok(Some(S::from_sample_(n?)))
        } else {
            Ok(None)
        }
    }
}

struct I32SampleReader<'wr, R: Read> {
    samples: WavSamples<'wr, R, i32>,
}

impl<'wr, R: Read, S> HoundSampleReader<S> for I32SampleReader<'wr, R>
where
    S: FromSample<i32>,
{
    fn read_sample(&mut self) -> Result<Option<S>, hound::Error> {
        if let Some(n) = self.samples.next() {
            Ok(Some(S::from_sample_(n?)))
        } else {
            Ok(None)
        }
    }
}

struct I16SampleReader<'wr, R: Read> {
    samples: WavSamples<'wr, R, i16>,
}

impl<'wr, R: Read, S> HoundSampleReader<S> for I16SampleReader<'wr, R>
where
    S: FromSample<i16>,
{
    fn read_sample(&mut self) -> Result<Option<S>, hound::Error> {
        if let Some(n) = self.samples.next() {
            Ok(Some(S::from_sample_(n?)))
        } else {
            Ok(None)
        }
    }
}

pub struct HoundAudioWriter<'ww, S>
where
    S: ToSample<f32> + ToSample<i32> + ToSample<i16>,
{
    hound_sample_writer: Box<dyn HoundSampleWriter<S> + 'ww>,
    number_of_channels: usize,
}

impl<'ww, S> HoundAudioWriter<'ww, S>
where
    S: ToSample<f32> + ToSample<i32> + ToSample<i16>,
{
    fn hound_sample_writer<W: Write + Seek>(
        writer: &'ww mut WavWriter<W>,
    ) -> Result<Box<dyn HoundSampleWriter<S> + 'ww>, HoundAudioError> {
        let spec = writer.spec();
        Ok(match spec.sample_format {
            hound::SampleFormat::Float => match spec.bits_per_sample {
                32 => Box::new(F32SampleWriter { writer }),
                _ => {
                    return Err(HoundAudioError::UnsupportedAudioFormat);
                }
            },
            hound::SampleFormat::Int => match spec.bits_per_sample {
                22 | 32 => Box::new(I32SampleWriter { writer }),
                8 | 16 => Box::new(I16SampleWriter { writer }),
                _ => {
                    // Note: until 3.4.0, Hound only supports 8, 16, 24, 32 bits/sample.
                    // Something else (e.g. 12 bits) would result in an error while writing
                    // a sample, so it does not make sense to allow this at this point.
                    return Err(HoundAudioError::UnsupportedAudioFormat);
                }
            },
        })
    }

    pub fn new<W: Write + Seek>(writer: &'ww mut WavWriter<W>) -> Result<Self, HoundAudioError> {
        let spec = writer.spec();
        let hound_sample_writer = Self::hound_sample_writer(writer)?;
        Ok(Self {
            hound_sample_writer,
            number_of_channels: spec.channels as usize,
        })
    }
}

impl<'ww, S> AudioWriter<S> for HoundAudioWriter<'ww, S>
where
    S: ToSample<f32> + ToSample<i32> + ToSample<i16> + Copy,
{
    type Err = hound::Error;

    fn write_buffer(&mut self, inputs: &AudioBufferIn<S>) -> Result<(), Self::Err> {
        assert_eq!(inputs.number_of_channels(), self.number_of_channels);
        assert!(self.number_of_channels > 0);
        let length = inputs.number_of_frames();

        let mut frame_index = 0;
        while frame_index < length {
            for input in inputs.channels().iter() {
                self.hound_sample_writer.write_sample(input[frame_index])?;
            }
            frame_index += 1;
        }

        self.hound_sample_writer.flush()
    }
}

trait HoundSampleWriter<S> {
    fn write_sample(&mut self, sample: S) -> Result<(), hound::Error>;
    fn flush(&mut self) -> Result<(), hound::Error>;
}

struct F32SampleWriter<'ww, W>
where
    W: Write + Seek,
{
    writer: &'ww mut WavWriter<W>,
}

impl<'ww, S, W> HoundSampleWriter<S> for F32SampleWriter<'ww, W>
where
    S: ToSample<f32>,
    W: Write + Seek,
{
    fn write_sample(&mut self, sample: S) -> Result<(), hound::Error> {
        self.writer.write_sample::<f32>(sample.to_sample_())
    }
    fn flush(&mut self) -> Result<(), hound::Error> {
        self.writer.flush()
    }
}

struct I32SampleWriter<'ww, W>
where
    W: Write + Seek,
{
    writer: &'ww mut WavWriter<W>,
}

impl<'ww, S, W> HoundSampleWriter<S> for I32SampleWriter<'ww, W>
where
    S: ToSample<i32>,
    W: Write + Seek,
{
    fn write_sample(&mut self, sample: S) -> Result<(), hound::Error> {
        self.writer.write_sample::<i32>(sample.to_sample_())
    }

    fn flush(&mut self) -> Result<(), hound::Error> {
        self.writer.flush()
    }
}

struct I16SampleWriter<'ww, W>
where
    W: Write + Seek,
{
    writer: &'ww mut WavWriter<W>,
}

impl<'ww, S, W> HoundSampleWriter<S> for I16SampleWriter<'ww, W>
where
    S: ToSample<i16>,
    W: Write + Seek,
{
    fn write_sample(&mut self, sample: S) -> Result<(), hound::Error> {
        self.writer.write_sample::<i16>(sample.to_sample_())
    }

    fn flush(&mut self) -> Result<(), hound::Error> {
        self.writer.flush()
    }
}
