use num_traits::Zero;
use std::mem;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AudioBuffer<F> {
    // Invariant: channels is not empty.
    channels: Vec<Vec<F>>,
}

impl<F> AudioBuffer<F> {
    pub fn zero(number_of_channels: usize, buffer_size: usize) -> Self
    where
        F: Zero,
    {
        let mut buffers = Vec::with_capacity(number_of_channels);
        for _ in 0..number_of_channels {
            let mut buffer = Vec::with_capacity(buffer_size);
            for _ in 0..buffer_size {
                buffer.push(F::zero());
            }
            buffers.push(buffer);
        }
        Self { channels: buffers }
    }

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

    pub fn channels(&self) -> &Vec<Vec<F>> {
        &self.channels
    }

    pub fn append_chunk(&mut self, chunk: &[&[F]])
    where
        F: Clone,
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

    pub fn inner(self) -> Vec<Vec<F>> {
        self.channels
    }

    pub fn as_slices<'a>(&'a self) -> Vec<&[F]> {
        self.channels
            .iter()
            .map(|element| element.as_slice())
            .collect()
    }

    pub fn as_mut_slices<'a>(&'a mut self) -> Vec<&mut [F]> {
        self.channels
            .iter_mut()
            .map(|element| element.as_mut_slice())
            .collect()
    }

    pub fn split(mut self, chunk_size: usize) -> Vec<Self> {
        assert!(chunk_size > 0);

        let number_of_samples = self.channels[0].len();

        let result_len = number_of_samples / chunk_size
            + if number_of_samples % chunk_size == 0 {
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
                if chunk.len() == chunk_size {
                    result[chunk_index].push(mem::replace(&mut chunk, Vec::new()));
                    chunk_index += 1;
                }
            }
            if !chunk.is_empty() {
                result[chunk_index].push(chunk);
            }
        }
        result.drain(..).map(AudioBuffer::from_channels).collect()
    }
}

#[macro_export]
macro_rules! audio_buffer {
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
        $crate::dev_utilities::chunk::AudioBuffer::from_channels(
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
fn append_chunk_works_as_expected() {
    let mut audio_buffer = AudioBuffer::new(3);
    let input = audio_buffer![[1, 2], [3, 4], [5, 6]];
    audio_buffer.append_chunk(input.as_slices().as_ref());
    assert_eq!(audio_buffer.channels()[0], vec![1, 2]);
    assert_eq!(audio_buffer.channels()[1], vec![3, 4]);
    assert_eq!(audio_buffer.channels()[2], vec![5, 6]);
}

#[test]
fn chunk_works_with_dividing_input_length() {
    let input = audio_buffer![[0, 1, 2, 3], [5, 6, 7, 8]];
    let observed = input.split(2);
    assert_eq!(
        observed,
        vec![audio_buffer![[0, 1], [5, 6]], audio_buffer![[2, 3], [7, 8]]]
    )
}

#[test]
fn chunk_works_with_non_dividing_input_length() {
    let input = audio_buffer![[0, 1, 2, 3, 4], [5, 6, 7, 8, 9]];
    let observed = input.split(2);
    assert_eq!(
        observed,
        vec![
            audio_buffer![[0, 1], [5, 6]],
            audio_buffer![[2, 3], [7, 8]],
            audio_buffer![[4], [9]]
        ]
    )
}

pub fn buffers_as_slice<'a, F>(buffers: &'a Vec<Vec<F>>, slice_len: usize) -> Vec<&'a [F]> {
    buffers.iter().map(|b| &b[0..slice_len]).collect()
}

pub fn buffers_as_mut_slice<'a, F>(
    buffers: &'a mut Vec<Vec<F>>,
    slice_len: usize,
) -> Vec<&'a mut [F]> {
    buffers.iter_mut().map(|b| &mut b[0..slice_len]).collect()
}
