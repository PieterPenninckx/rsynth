#[macro_use] extern crate vst2;
#[macro_use] extern crate easyvst;
extern crate num_traits;
extern crate num;
extern crate asprim;
extern crate rvst_synth;
extern crate rand;

use num_traits::Float;
use asprim::AsPrim;

use vst2::buffer::AudioBuffer;
use vst2::plugin::{Category, Info, HostCallback};

use easyvst::*;
use rvst_synth::synthesizer::*;
use rvst_synth::voice::*;

easyvst!(ParamId, ExState, ExPlugin);

/// A struct containing all usable parameters

#[repr(usize)]
#[derive(Debug, Copy, Clone)]
pub enum ParamId {
	Pitch
}

#[derive(Default)]
struct ExState {
	pitch: f32
}

impl UserState<ParamId> for ExState {
	fn param_changed(&mut self, _host: &mut HostCallback, param_id: ParamId, val: f32) {
		use ParamId::*;
		match param_id {
			Pitch => self.pitch = 0f32
		}
	}

	fn format_param(&self, param_id: ParamId, val: f32) -> String {
		use ParamId::*;
		match param_id {
			Pitch => format!("{:.2}", val),
		}
	}
}

type ExPluginState = PluginState<ParamId, ExState>;

#[derive(Default)]
struct ExPlugin {
	state: ExPluginState,
}

#[allow(unused_variables)]
impl ExPlugin {
	fn process_one_channel<F: Float + AsPrim>(&mut self, input: &[F], output: &mut [F]) {
		
	}
}

impl EasyVst<ParamId, ExState> for ExPlugin {
	fn params() -> Vec<ParamDef> {
		vec![
			ParamDef::new("Gain", -48., 12., 0.),
		]
	}

	fn state(&self) -> &ExPluginState { &self.state }

	fn state_mut(&mut self) -> &mut ExPluginState { &mut self.state }

	fn get_info(&self) -> Info {
		Info {
			name: "easygain".to_string(),
			vendor: "easyvst".to_string(),
			unique_id: 0x3456DCBA,
			category: Category::Effect,

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
		
	}

	fn process_f<T: Float + AsPrim>(&mut self, buffer: AudioBuffer<T>) {
		// split out the input and output buffers into two vectors
		let (inputs, outputs) = buffer.split();

		// for each buffer, transform the samples
		for (input_buffer, output_buffer) in inputs.iter().zip(outputs) {
			self.process_one_channel(input_buffer, output_buffer);
		}
	}
}


#[allow(unused_mut, unused_variables)]
fn main(){
    let mut synth: rvst_synth::synthesizer::Synthesizer<MyVoice> = Synthesizer { 
									    	sample_rate: 48_000f64, 
									    	note_steal: StealMode::First, 
									    	voices: vec![] };


	pub struct MyVoice {
		pub state: VoiceState
	}

	impl Voice for MyVoice {

	    /// Do all our processing here
	    fn render_next<F: Float + AsPrim>(&mut self, input: &[F], output: &mut [F]) {
	    	// if no note is currently being played, just write zeroes.
	    	if self.current_note {

	    	}

	    	for o_sample in output {
	    		*o_sample = num::cast(rand::random::<f64>()).unwrap();
	    	}
	    }
	}
}