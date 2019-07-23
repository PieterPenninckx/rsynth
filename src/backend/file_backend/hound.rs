use super::AudioReader;
use hound::{Sample, WavReader, WavSamples};
use sample::conv::FromSample;
use std::io::Read;

pub trait HoundSampleReader<F> {
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
            n.map(|n| F::from_sample_(n)).ok()
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
            n.map(|n| F::from_sample_(n)).ok()
        } else {
            None
        }
    }
}

pub struct I16SampleReader<'wr, R: Read> {
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
