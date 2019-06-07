// An example of a software synthesizer using the JACK back-end.
// The code that is shared between all backends is in the `test_synth.rs` file.
//
// Compiling
// =========
// You can compile this example with
// ```
// cargo build --release --examples
// ```
// This generates a library that you can find
//
// * under `target/release/examples/libvst_synth.so` for linux
// * in the `target/release/examples/` folder for other operating systems.
//
// Running
// =======
//
// ## Under Linux
// Copy the `.so` file to a folder that is in the `VST_PATH` environment variable.
//
// ## Under Windows
// TODO
//
// ## Under MacOs
// TODO
// Note: the `rust-vst` repository contains a file `osx_vst_bundler.sh`, you will probably
// need this.
//
// ## Logging
// In order to enable logging, set the environment variable `RSYNTH_LOG_LEVEL` to
// one of the supported log levels.
// Recognized log levels are: 'off', 'error', 'warning', 'info', 'debug' and 'trace'.
//
// You can set the environment variable `RSYNTH_LOG_FILE` to the file name of the file in which
// you want to log.
//
// Note that these environment variables need to be visible to the host.
// Note that the example is also logging to a file in the realtime thread, which may cause clipping.

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

use rsynth::{
    output_mode::{
        OutputMode, Additive
    }, 
    event::{
        Timed,
        RawMidiEvent
    },
    backend::vst_backend::VstPlugin
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
