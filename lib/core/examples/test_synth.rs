#[macro_use]
extern crate vst2;
extern crate rsynth_core;
extern crate rand;
extern crate asprim;
extern crate num_traits;
extern crate num;

use vst2::plugin::{Category, Info, Plugin};
use vst2::buffer::{AudioBuffer, Inputs, Outputs}; 
use vst2::api::Events;
use rsynth_core::synthesizer::*;
use rsynth_core::voice::*;
use rsynth_core::utility::note::NoteData;
use num_traits::Float;
use asprim::AsPrim;

#[derive(Default)]
struct RSynthExample {
	synth: Synthesizer<Sound>,
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
		let voice = Voice { 
			pan: 0f32, 
			sound: Sound { }, 
			state: VoiceState::Off,
			note_data: NoteData::default()  };

		self.synth = Synthesizer::new()
						.voices(vec![voice; 6])
						.sample_rate(41_000f64)
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

}

/// The DSP stuff goes here
impl Renderable for Sound {
    #[allow(unused_variables)]
    fn render_next<F: Float + AsPrim, T> (&self, inputs: &mut Inputs<F>, outputs: &mut Outputs<F>, voice: &Voice<T>) where T: Renderable {
    	// for every output
    	for output in outputs.into_iter() {
    		// for each value in buffer
    		for sample in output {
    			*sample = *sample + ((rand::random::<f64>() / 4f64) * (voice.note_data.velocity as f64 / 127f64) ).as_();
    		}
    	}
    }
}

plugin_main!(RSynthExample); 