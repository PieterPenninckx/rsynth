use super::AudioReader;
use hound::{Sample, WavReader, WavSamples};
use std::io::Read;

pub trait FromSample<Source>: Copy {
    fn convert(source: Source) -> Self;
}

impl FromSample<f32> for f64 {
    #[inline(always)]
    fn convert(source: f32) -> Self {
        source as f64
    }
}

impl FromSample<f32> for f32 {
    #[inline(always)]
    fn convert(source: f32) -> Self {
        source
    }
}

impl FromSample<f32> for i32 {
    #[inline(always)]
    fn convert(source: f32) -> Self {
        (source * (i32::max_value() as f32)) as i32
    }
}

impl FromSample<f32> for i16 {
    #[inline(always)]
    fn convert(source: f32) -> Self {
        (source * (i16::max_value() as f32)) as i16
    }
}

impl FromSample<i32> for f64 {
    #[inline(always)]
    fn convert(source: i32) -> Self {
        source as f64 / (-(i32::min_value()) as f64)
    }
}

impl FromSample<i32> for f32 {
    #[inline(always)]
    fn convert(source: i32) -> Self {
        source as f32 / (-(i32::min_value()) as f32)
    }
}

// Etc. etc.
// TODO: consider using the `Sample` crate for this. https://github.com/RustAudio/sample

pub trait HoundSampleReader<F>
where
    F: Copy,
{
    fn read_sample(&mut self) -> Option<F>;
}

pub struct F32SampleReader<'wr, R: Read> {
    samples: WavSamples<'wr, R, f32>,
}

impl<'wr, R: Read, F> HoundSampleReader<F> for F32SampleReader<'wr, R>
where
    F: FromSample<f32>,
{
    fn read_sample(&mut self) -> Option<F> {
        if let Some(n) = self.samples.next() {
            n.map(|n| F::convert(n)).ok()
        } else {
            None
        }
    }
}

pub struct I32SampleReader<'wr, R: Read> {
    samples: WavSamples<'wr, R, i32>,
}

impl<'wr, R: Read, F> HoundSampleReader<F> for I32SampleReader<'wr, R>
where
    F: FromSample<i32>,
{
    fn read_sample(&mut self) -> Option<F> {
        if let Some(n) = self.samples.next() {
            n.map(|n| F::convert(n)).ok()
        } else {
            None
        }
    }
}

pub struct I16SampleReader<'wr, R: Read> {
    samples: WavSamples<'wr, R, i16>,
}

pub trait HoundSample<'wr, R: Read>: Copy {
    fn reader(r: &'wr mut WavReader<R>) -> Box<dyn HoundSampleReader<Self> + 'wr>;
}

impl<'wr, R: Read> HoundSample<'wr, R> for f32 {
    fn reader(r: &'wr mut WavReader<R>) -> Box<dyn HoundSampleReader<Self> + 'wr> {
        let spec = r.spec();
        match spec.sample_format {
            hound::SampleFormat::Float => {
                assert_eq!(spec.bits_per_sample, 32); // TODO: Better error handling.
                Box::new(F32SampleReader {
                    samples: r.samples(),
                })
            }
            hound::SampleFormat::Int => match spec.bits_per_sample {
                32 => Box::new(I32SampleReader {
                    samples: r.samples(),
                }),
                _ => unimplemented!(),
            },
        }
    }
}

pub struct HoundAudioReader<F> {
    reader: Box<dyn HoundSampleReader<F>>,
    number_of_channels: usize,
    sample_rate: f64,
}

impl<F> HoundAudioReader<F> {
    pub fn new<'wr, R: Read>(reader: &'wr mut WavReader<R>) -> Option<Self>
    where
        F: HoundSample<'wr, R>,
    {
        let spec = reader.spec();

        let number_of_channels = spec.channels as usize;
        let sample_rate = spec.sample_rate as f64;
        unimplemented!();
    }
}

impl<F> AudioReader<F> for HoundAudioReader<F> {
    fn number_of_channels(&self) -> usize {
        self.number_of_channels
    }

    fn fill_buffer(&mut self, output: &mut [&mut [F]]) -> usize {
        unimplemented!()
    }
}
