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
	synth: Synthesizer<Sound>,
	state: ExPluginState,
}

impl EasyVst<ParamId, ExState> for ExPlugin {
	fn params() -> Vec<ParamDef> {
		vec![
			ParamDef::new("Pitch", -48., 12., 0.),
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
		self.synth = Synthesizer { 
				    	sample_rate: 48_000f64, 
				    	note_steal: StealMode::First, 
				    	voices: vec![] };

	}

	fn process_f<T: Float + AsPrim>(&mut self, buffer: AudioBuffer<T>) {
		let (inputs, outputs) = buffer.split();

		let mut s = Sound { state: VoiceState::On };
		for (input_buffer, output_buffer) in outputs.iter().zip(inputs) {
			s.render_next::<T, Sound>(input_buffer, output_buffer);
            // &self.synth.render_channel(input_buffer, output_buffer);	
            // Not working :(
        }
	}
}


/// The DSP stuff goes here
pub struct Sound {
	pub state: VoiceState
}

impl Renderable for Sound {

    /// Do all our DSP stuff here
    #[allow(unused_variables)]
    fn render_next<F: Float + AsPrim, T> (&mut self, input: &[F], output: &mut [F]) where T: Renderable {
    	for o_sample in output {
    		*o_sample = num::cast(rand::random::<f64>()).unwrap();
    	}
    }
}