#[macro_use]
extern crate vst;
extern crate rsynth;
extern crate rand;
extern crate asprim;
extern crate num_traits;

use vst::plugin::{Category, Info, Plugin};
use vst::buffer::{AudioBuffer, Inputs, Outputs}; 
use vst::api::Events;
use rsynth::synth::*;
use rsynth::voice::*;
use num_traits::Float;
use asprim::AsPrim;
use rand::{thread_rng, Rng};
use std::cell::Cell;

const DEFAULT_SAMPLE_RATE: f64 = 48_000f64;

// The total number of samples to pre-calculate
// This is like recording a sample of white noise and then
// using it as an oscillator.  It saves on CPU overhead by 
// preventing us from having to use a random function each sample.
static SAMPLE_SIZE: usize = 65536;
static AMPLIFY_MULTIPLIER: f32 = 0.2;

#[derive(Default)]
struct RSynthExample {
	synth: Synth<Sound>,
}

impl Plugin for RSynthExample {
    fn get_info(&self) -> Info {
        Info {
            name: "RSynth Example".to_string(),
            unique_id: 1234, // Used by hosts to differentiate between plugins.
            category: Category::Synth,
            inputs: 0,
            outputs: 2,
            ..Default::default()
        }
    }

    fn init(&mut self) {
        // generate our random sample
        let mut rng = thread_rng();
        let samples: Vec<f32> = rng.gen_iter::<f32>().take(SAMPLE_SIZE).collect::<Vec<f32>>();  
        let sound = Sound { sample_count: samples.len(), white_noise: samples, position: Cell::new(0usize) };

		let voice = VoiceBuilder::new_with_sound(sound).sample_rate(DEFAULT_SAMPLE_RATE).finalize();

		self.synth = Synth::new()
						.voices(vec![voice; 6])
						.sample_rate(DEFAULT_SAMPLE_RATE) // TODO: use host sample rate
						.finalize();
	}


    fn process_events(&mut self, events: &Events) {
		// send midi data, etc.
		self.synth.process_events(events);
	}

	fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
		// render our audio
		self.synth.render_next::<f32>(buffer);
	}
}


#[derive(Clone)]
pub struct Sound {
    white_noise: Vec<f32>,
    sample_count: usize,
    // we use cell here for interior mutability
    position: Cell<usize>
}

/// The DSP stuff goes here
impl Renderable for Sound {
    #[allow(unused_variables)]
    fn render_next<F: Float + AsPrim, T> (&self, inputs: &mut Inputs<F>, outputs: &mut Outputs<F>, voice: &Voice<T>) where T: Renderable {
    	// for every output
    	for output in outputs.into_iter() {
    		// for each value in buffer
    		for (i, sample) in output.into_iter().enumerate() {
                // Increment the position of our sound sample.
                // We loop this easily by using modulo.
                self.position.set((self.position.get() + 1) % self.sample_count);

                // Our random function only generates from 0 - 1.  We can make
                // it distribute equally by multiplying by 2 and subtracting by 1.
                let r = 2f32 *(self.white_noise[self.position.get()]) - 1f32;

                // Set our output buffer
    			*sample = *sample + ( (r * AMPLIFY_MULTIPLIER) * (voice.note_data.velocity as f32 / 127f32) ).as_();
    		}
    	}
    }
}

plugin_main!(RSynthExample); 