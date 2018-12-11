#[macro_use]
extern crate vst;
#[macro_use]
extern crate log;
extern crate simplelog;
extern crate asprim;
extern crate num_traits;
extern crate rand;
#[macro_use]
extern crate rsynth;

mod test_synth;
use test_synth::*;

use rsynth::polyphony::{Polyphonic, SimpleVoiceStealer};
use rsynth::backend::vst_backend::VstPlugin;

use vst::plugin::Category;

impl VstPlugin for Sound {
    const PLUGIN_ID: i32 = 123;
    const CATEGORY: Category = Category::Synth;
}

vst_init!(
    fn init() -> Polyphonic<Sound, SimpleVoiceStealer<Sound>> {
        let mut voices = Vec::new();
        for _ in 0 .. 6 {
            voices.push(Sound::default());
        }
        let p = Polyphonic::new(SimpleVoiceStealer::new(), voices);
        return p;
    }
);