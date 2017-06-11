#[macro_use] extern crate vst2;
#[macro_use] extern crate easyvst;
extern crate num_traits;
extern crate num;
extern crate asprim;
extern crate rvst_synth;
extern crate rand;

use num_traits::Float;
use asprim::AsPrim;

use vst2::buffer::{AudioBuffer, Inputs, Outputs}; 
use vst2::plugin::{Category, Info, HostCallback};

use easyvst::*;
use rvst_synth::synthesizer::*;
use rvst_synth::voice::*;
use rvst_synth::utility::*;

easyvst!(ParamId, ExState, ExPlugin);

/// A struct containing all usable parameters
#[repr(usize)]
#[derive(Debug, Copy, Clone)]
pub enum ParamId {
	Pitch
}

#[derive(Default)]
struct ExState {
	// the raw pan amount between -1 and 1
	pan: f32,
	// a stereo pan tuple representing amp for pan
	pan_raw: (f32, f32)
}

impl UserState<ParamId> for ExState {
	fn param_changed(&mut self, _host: &mut HostCallback, param_id: ParamId, val: f32) {
		use ParamId::*;
		match param_id {
			Panning => {
				self.pan_raw = constant_power_pan(val);
				self.pan = val;
			}
		}
	}

	fn format_param(&self, param_id: ParamId, val: f32) -> String {
		use ParamId::*;
		match param_id {
			Panning => format!("{:.2}", val),
		}
	}
}

type ExPluginState = PluginState<ParamId, ExState>;

#[derive(Default)]
struct ExPlugin {
	synth: Synthesizer<Sound>,
	state: ExPluginState,
}

impl EasyVst<ParamId, ExState> for ExPlugin {
	fn params() -> Vec<ParamDef> {
		vec![
			ParamDef::new("Panning", -1., 1., 0.),
		]
	}

	fn state(&self) -> &ExPluginState { &self.state }

	fn state_mut(&mut self) -> &mut ExPluginState { &mut self.state }

	fn get_info(&self) -> Info {
		Info {
			name: "sinesynth".to_string(),
			vendor: "sinesynth".to_string(),
			unique_id: 0x3456DFFA,
			category: Category::Synth,

			inputs: 2,
			outputs: 2,
			parameters: 1,

			..Info::default()
		}
	}

	fn new(state: ExPluginState) -> Self {
		let mut p: ExPlugin = Default::default();
		p.state = state;
		p
	}

	fn init(&mut self) {
		let voice = Voice { pan: 0f32, sound: Sound { }, state: VoiceState::Off };

		self.synth = Synthesizer::new()
						.voices(vec![voice])
						.sample_rate(41_000f64)
						.finalize();
	}

	fn process_f<T: Float + AsPrim>(&mut self, buffer: &mut AudioBuffer<T>) {
		// set the panning amount from our state object
		self.synth.pan_raw = self.state.user_state.pan_raw;

		// render our audio
		self.synth.render_next::<T>(buffer);
	}
}


/// The DSP stuff goes here
pub struct Sound {

}

impl Renderable for Sound {

    /// Do all our DSP stuff here
    #[allow(unused_variables)]
    fn render_next<F: Float + AsPrim, T> (&self, inputs: &mut Inputs<F>, outputs: &mut Outputs<F>, voice: &Voice<T>) where T: Renderable {
    	// for every output
    	for output in outputs.into_iter() {
    		// for each value in buffer
    		for sample in output {
    			*sample = *sample + rand::random::<f64>().as_();
    		}
    	}
    }
}