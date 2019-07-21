// An example of a software synthesizer using the JACK back-end.
// The code that is shared between all backends is in the `test_synth.rs` file.
//
// Compiling
// =========
// You can compile this example with
// ```
// cargo build --release --examples --features backend-vst
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

#[cfg(feature = "backend-vst")]
#[macro_use]
extern crate vst;
#[macro_use]
extern crate log;
extern crate asprim;
extern crate num_traits;
extern crate rand;
#[macro_use]
extern crate rsynth;

mod test_synth;
use test_synth::*;

#[cfg(feature = "backend-vst")]
use rsynth::backend::vst_backend::VstPluginMeta;

#[cfg(feature = "backend-vst")]
use vst::plugin::Category;

#[cfg(feature = "backend-vst")]
impl VstPluginMeta for NoisePlayer {
    const PLUGIN_ID: i32 = 123;
    const CATEGORY: Category = Category::Synth;
}

#[rustfmt::skip::macros(vst_init)]
#[cfg(feature = "backend-vst")]
vst_init!(
    fn init() -> NoisePlayer {
        NoisePlayer::new()
    }
);
