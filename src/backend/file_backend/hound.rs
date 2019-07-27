use super::{AudioReader, AudioWriter};
use hound::{Sample, WavReader, WavSamples, WavWriter};
use sample::conv::{FromSample, ToSample};
use std::io::{Read, Seek, Write};

trait HoundSampleReader<F> {
    fn read_sample(&mut self) -> Option<F>;
}

struct F32SampleReader<'wr, R: Read> {
    samples: WavSamples<'wr, R, f32>,
}

impl<'wr, R: Read, F> HoundSampleReader<F> for F32SampleReader<'wr, R>
where
    F: FromSample<f32>,
{
    fn read_sample(&mut self) -> Option<F> {
        if let Some(n) = self.samples.next() {
            n.map(|n| F::from_sample_(n)).ok()
        } else {
            None
        }
    }
}

struct I32SampleReader<'wr, R: Read> {
    samples: WavSamples<'wr, R, i32>,
}

impl<'wr, R: Read, F> HoundSampleReader<F> for I32SampleReader<'wr, R>
where
    F: FromSample<i32>,
{
    fn read_sample(&mut self) -> Option<F> {
        if let Some(n) = self.samples.next() {
            n.map(|n| F::from_sample_(n)).ok()
        } else {
            None
        }
    }
}

struct I16SampleReader<'wr, R: Read> {
    samples: WavSamples<'wr, R, i16>,
}

impl<'wr, R: Read, F> HoundSampleReader<F> for I16SampleReader<'wr, R>
where
    F: FromSample<i16>,
{
    fn read_sample(&mut self) -> Option<F> {
        if let Some(n) = self.samples.next() {
            n.map(|n| F::from_sample_(n)).ok()
        } else {
            None
        }
    }
}

pub struct HoundAudioReader<'wr, F>
where
    F: FromSample<f32> + FromSample<i32> + FromSample<i16>,
{
    hound_sample_reader: Box<dyn HoundSampleReader<F> + 'wr>,
    number_of_channels: usize,
    sample_rate: f64,
}

impl<'wr, F> HoundAudioReader<'wr, F>
where
    F: FromSample<f32> + FromSample<i32> + FromSample<i16>,
{
    fn reader<R: Read>(r: &'wr mut WavReader<R>) -> Box<dyn HoundSampleReader<F> + 'wr> {
        let spec = r.spec();
        match spec.sample_format {
            hound::SampleFormat::Float => match spec.bits_per_sample {
                32 => Box::new(F32SampleReader {
                    samples: r.samples(),
                }),
                _ => {
                    // TODO: better error handling.
                    panic!("Of all the float type, only 32 bits floats are supported.");
                }
            },
            hound::SampleFormat::Int => match spec.bits_per_sample {
                32 => Box::new(I32SampleReader {
                    samples: r.samples(),
                }),
                16 => Box::new(I16SampleReader {
                    samples: r.samples(),
                }),
                _ => {
                    // TODO: better error handling.
                    panic!("Of all the int types, only 16 bit and 32 bit integers are supported.");
                }
            },
        }
    }

    pub fn new<R: Read>(reader: &'wr mut WavReader<R>) -> Option<Self> {
        let spec = reader.spec();

        let number_of_channels = spec.channels as usize;
        let sample_rate = spec.sample_rate as f64;
        let hound_sample_reader = Self::reader(reader);
        Some(Self {
            number_of_channels,
            sample_rate,
            hound_sample_reader,
        })
    }
}

impl<'wr, F> AudioReader<F> for HoundAudioReader<'wr, F>
where
    F: FromSample<f32> + FromSample<i32> + FromSample<i16>,
{
    fn number_of_channels(&self) -> usize {
        self.number_of_channels
    }

    fn fill_buffer(&mut self, outputs: &mut [&mut [F]]) -> usize {
        assert_eq!(outputs.len(), self.number_of_channels());
        assert!(self.number_of_channels() > 0);
        let length = outputs[0].len();
        for output in outputs.iter() {
            assert_eq!(output.len(), length);
        }
        let mut frame_index = 0;
        while frame_index < length {
            for output in outputs.iter_mut() {
                if let Some(sample) = self.hound_sample_reader.read_sample() {
                    output[frame_index] = sample;
                } else {
                    return frame_index;
                }
            }
            frame_index += 1;
        }
        return frame_index;
    }
}

pub struct HoundAudioWriter<'ww, F>
where
    F: ToSample<f32> + ToSample<i32> + ToSample<i16>,
{
    hound_sample_writer: Box<dyn HoundSampleWriter<F> + 'ww>,
    number_of_channels: usize,
    sample_rate: f64,
}

impl<'ww, F> HoundAudioWriter<'ww, F>
where
    F: ToSample<f32> + ToSample<i32> + ToSample<i16>,
{
    fn hound_sample_writer<W: Write + Seek>(
        writer: &'ww mut WavWriter<W>,
    ) -> Box<dyn HoundSampleWriter<F> + 'ww> {
        let spec = writer.spec();
        match spec.sample_format {
            hound::SampleFormat::Float => match spec.bits_per_sample {
                32 => Box::new(F32SampleWriter { writer }),
                _ => {
                    // TODO: better error handling.
                    panic!("Of all the float type, only 32 bits floats are supported.");
                }
            },
            hound::SampleFormat::Int => match spec.bits_per_sample {
                32 => Box::new(I32SampleWriter { writer }),
                16 => Box::new(I16SampleWriter { writer }),
                _ => {
                    // TODO: better error handling.
                    panic!("Of all the int types, only 16 bit and 32 bit integers are supported.");
                }
            },
        }
    }

    pub fn new<W: Write + Seek>(writer: &'ww mut WavWriter<W>) -> Self {
        let spec = writer.spec();
        let hound_sample_writer = Self::hound_sample_writer(writer);
        Self {
            hound_sample_writer,
            number_of_channels: spec.channels as usize,
            sample_rate: spec.sample_rate as f64,
        }
    }
}

impl<'ww, F> AudioWriter<F> for HoundAudioWriter<'ww, F>
where
    F: ToSample<f32> + ToSample<i32> + ToSample<i16> + Copy,
{
    fn write_buffer(&mut self, inputs: &[&[F]]) {
        assert_eq!(inputs.len(), self.number_of_channels);
        assert!(self.number_of_channels > 0);
        let length = inputs[0].len();
        for input in inputs.iter() {
            assert_eq!(inputs.len(), length);
        }

        let mut frame_index = 0;
        while frame_index < length {
            for input in inputs.iter() {
                self.hound_sample_writer.write_sample(input[frame_index]);
            }
            frame_index += 1;
        }
        self.hound_sample_writer.flush();
    }
}

trait HoundSampleWriter<F> {
    fn write_sample(&mut self, sample: F);
    fn flush(&mut self);
}

struct F32SampleWriter<'ww, W>
where
    W: Write + Seek,
{
    writer: &'ww mut WavWriter<W>,
}

impl<'ww, F, W> HoundSampleWriter<F> for F32SampleWriter<'ww, W>
where
    F: ToSample<f32>,
    W: Write + Seek,
{
    fn write_sample(&mut self, sample: F) {
        self.writer.write_sample::<f32>(sample.to_sample_());
    }
    fn flush(&mut self) {
        self.writer.flush();
    }
}

struct I32SampleWriter<'ww, W>
where
    W: Write + Seek,
{
    writer: &'ww mut WavWriter<W>,
}

impl<'ww, F, W> HoundSampleWriter<F> for I32SampleWriter<'ww, W>
where
    F: ToSample<i32>,
    W: Write + Seek,
{
    fn write_sample(&mut self, sample: F) {
        self.writer.write_sample::<i32>(sample.to_sample_());
    }

    fn flush(&mut self) {
        self.writer.flush();
    }
}

struct I16SampleWriter<'ww, W>
where
    W: Write + Seek,
{
    writer: &'ww mut WavWriter<W>,
}

impl<'ww, F, W> HoundSampleWriter<F> for I16SampleWriter<'ww, W>
where
    F: ToSample<i16>,
    W: Write + Seek,
{
    fn write_sample(&mut self, sample: F) {
        self.writer.write_sample::<i16>(sample.to_sample_());
    }

    fn flush(&mut self) {
        self.writer.flush();
    }
}