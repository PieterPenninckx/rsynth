extern crate rvst_synth;
extern crate vst2;
extern crate asprim;
extern crate num_traits;

use self::asprim::AsPrim;
use self::vst2::buffer::AudioBuffer;
use self::num_traits::Float;

use rvst_synth::synthesizer::*;
use rvst_synth::voice::*;

#[allow(unused_mut, unused_variables)]
fn main(){
    let mut synth: Synthesizer<MyVoice> = Synthesizer { sample_rate: 48_000f64, note_steal: StealMode::First, voices: vec![] };


	pub struct MyVoice {

	}

	impl Voice for MyVoice {

	    fn note_on(&self, note_data: &NoteData) {

	    }

	    fn note_off(&self) {

	    }

	    fn render_next<T: Float + AsPrim>(&self, buffer: &mut AudioBuffer<T>) {

	    }

	    fn get_note(&self) ->  Option<u8> {
	    	unimplemented!();
	    }
	}
}