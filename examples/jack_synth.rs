// An example of a software synthesizer using the JACK back-end.
// The code that is shared between all backends is in the `test_synth.rs` file.
//
// Compiling
// =========
// You can compile this example with
// ```
// cargo build --release --examples --features backend-jack
// ```
// This generates a standalone application that you can find
//
// * in `target/release/examples/jack_synth` when you're using Linux
// * under the `target/release/examples/` folder when you're using Windows or MacOs
//
// Running
// =======
//
// In order to run, you need three steps. Below, we discuss for each platform
// (only Linux for now) how you can do this.
//
// 1. Start the `jack` daemon.
// 2. Start the application generated during compiling.
// 3. Connect the audio output of the synthesizer to the "system" audio input
// 4. (Optionally) start a midi keyboard simulator
// 5. Connect the midi keyboard output to the synthesizer midi input
// 6. Start making some noise!
//
// ## Running under Linux
// I think the easiest way is to use `qjackctl` and maybe `jack-keyboard`.
// Then the steps become
//
// 1. Start `qjackctl`. Optionally configure some stuff. Click the "start" button.
// 2. Start the synthesizer generated during compiling. This needs to be done after
//    the jack daemon was started in step 1 because it automatically registers its ports upon
//    startup.
// 3. In Qjackctl, click on the "Connect" button. Under the "Audio" tab, connect the
//    synthesizer to the system.
// 4. (optionally) start `jack-keyboard`.
// 5. In Qjackctl, in the "Connections" window, under the "Midi" tab, connect the
//    midi keyboard to the synthesizer.
// 6. Press keys on the midi keyboard.
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
extern crate log;
extern crate asprim;
extern crate num_traits;
extern crate rand;
extern crate rsynth;

mod test_synth;
use test_synth::*;

#[cfg(feature = "backend-jack")]
use rsynth::backend::jack_backend::run;

#[cfg(feature = "backend-jack")]
fn main() {
    run(NoisePlayer::new());
}

#[cfg(not(feature = "backend-jack"))]
fn main() {
    println!("This example was compiled without support for jack.");
    println!("Compile with passing `--features backend-jack`");
    println!("as parameter to `cargo`.");
}
