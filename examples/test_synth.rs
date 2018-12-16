use asprim::AsPrim;
use num_traits::Float;
use rand::{thread_rng, Rng};
use rsynth::middleware::polyphony::Voice;
use rsynth::backend::{Plugin, Event, RawMidiEvent, output_mode::OutputMode};
use std::fs::File;

// The total number of samples to pre-calculate
// This is like recording a sample of white noise and then
// using it as an oscillator.  It saves on CPU overhead by
// preventing us from having to use a random function each sample.
static SAMPLE_SIZE: usize = 65536;
static AMPLIFY_MULTIPLIER: f32 = 0.2;

#[derive(Clone)]
pub struct Sound<M> {
    white_noise: Vec<f32>,
    sample_count: usize,
    position: usize,
    velocity: u8,
    is_playing: bool,
    mode: M
}

impl<M> Default for Sound<M> where M: OutputMode {
    fn default() -> Self {
        // You can use the `log` crate for debugging purposes.
        trace!("");
        let mut rng = thread_rng();
        let samples: Vec<f32> = rng
            .gen_iter::<f32>()
            .take(SAMPLE_SIZE)
            .collect::<Vec<f32>>();
        Sound {
            sample_count: samples.len(),
            white_noise: samples,
            position: 0,
            velocity: 0,
            is_playing: false,
            mode: M::default()
        }
    }
}

/// The DSP stuff goes here
impl<'e, U, M> Plugin<Event<RawMidiEvent<'e>, U>> for Sound<M>
where M: OutputMode
{
    const NAME: &'static str = "RSynth Example";
    const MAX_NUMBER_OF_AUDIO_INPUTS: usize = 0;
    const MAX_NUMBER_OF_AUDIO_OUTPUTS: usize = 2;

    fn audio_input_name(index: usize) -> String {
        // You can use the `log` crate for debugging purposes.
        trace!("index: {}", index);
        "".to_string()
        // Calling this would be a bug in the host.
    }

    fn audio_output_name(index: usize) -> String {
        trace!("index: {}", index);
        match index {
            0 => "left".to_string(),
            1 => "right".to_string(),
            _ => {
                "".to_string()
                // If we get at this point, this would indicate a bug in the host.
            }
        }
    }

    fn set_sample_rate(&mut self, _sample_rate: f64) {
        // We are not doing anything with this right now.
    }

    #[allow(unused_variables)]
    fn render_buffer<F>(&mut self, inputs: &[&[F]], outputs: &mut[&mut[F]])
        where F: Float + AsPrim
    {
        assert_eq!(2, outputs.len());
        // for every output
        for output in outputs {
            // for each value in buffer
            for (i, sample) in output.iter_mut().enumerate() {
                // Increment the position of our sound sample.
                // We loop this easily by using modulo.
                self.position = (self.position + 1) % self.sample_count;

                // Our random function only generates from 0 - 1.  We can make
                // it distribute equally by multiplying by 2 and subtracting by 1.
                let r = 2f32 * (self.white_noise[self.position]) - 1f32;

                let value : F = ((r * AMPLIFY_MULTIPLIER) * (self.velocity as f32 / 127f32)).as_();

                // Set our output buffer
                // This works both in a monophonic context and a polyphonic context.
                M::set(sample, value);
            }
        }
    }

    fn handle_event(&mut self, event: &Event<RawMidiEvent<'e>, U>) {
        trace!("event"); // TODO: Should events implement Debug?
        if let &Event::Timed {samples: _samples, event: ref e} = event {
            let state_and_chanel = e.data[0];
            if state_and_chanel & 0xF0 == 0x90 {
                self.is_playing = true;
                self.velocity = e.data[2];
            }
            if state_and_chanel & 0xF0 == 0x80 {
                self.velocity = 0;
                self.is_playing = false;
            }
        }
    }
}

// This enables using it in a polyphonic context.
impl<M> Voice for Sound<M> {
    fn is_playing(&self) -> bool {
        self.is_playing
    }
}
