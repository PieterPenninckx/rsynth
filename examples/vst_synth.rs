#[macro_use]
extern crate vst;
#[macro_use]
extern crate log;
extern crate asprim;
extern crate num_traits;
extern crate rand;
extern crate simplelog;
#[macro_use]
extern crate rsynth;

mod test_synth;
use test_synth::*;

use rsynth::backend::{
    output_mode::{Additive, OutputMode},
    event::{Timed, RawMidiEvent},
    vst_backend::VstPlugin,
};
use rsynth::middleware::polyphony::{Polyphonic, SimpleVoiceStealer};

use vst::plugin::Category;

impl<M> VstPlugin for Sound<M>
where
    M: OutputMode,
{
    const PLUGIN_ID: i32 = 123;
    const CATEGORY: Category = Category::Synth;
}

vst_init!(
    fn init() -> Polyphonic<Sound<Additive>, SimpleVoiceStealer<Sound<Additive>>, Timed<RawMidiEvent>> {
        initialize_logging();

        let mut voices = Vec::new();
        for _ in 0..6 {
            voices.push(Sound::default());
        }
        let p = Polyphonic::new(SimpleVoiceStealer::new(), voices);
        return p;
    }
);
