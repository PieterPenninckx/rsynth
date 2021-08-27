// An example of a software synthesizer using the offline ("combined") back-end.
// The code that is shared between all backends is in the `example_synth.rs` file.
//
// Compiling
// =========
// You can compile this example with
// ```bash
// cargo build --release --examples --features backend-combined-midly-0-5,backend-combined-wav-0-6
// ```
// This generates a standalone application that you can find
//
// * in `target/release/examples/offline_synth` when you're using Linux
// * under the `target/release/examples/` folder when you're using Windows or MacOs
//
#[macro_use]
extern crate log;
#[cfg(feature = "backend-combined-midly-0-5")]
extern crate midly_0_5;
extern crate num_traits;
extern crate rsynth;

mod example_synth;
use example_synth::*;

#[cfg(feature = "backend-combined-midly-0-5")]
use midly_0_5::Smf;
#[cfg(feature = "backend-combined")]
use rsynth::backend::combined::dummy::{AudioDummy, MidiDummy};
use std::fs::OpenOptions;
use std::{env, fs};

#[cfg(all(
    feature = "backend-combined-midly-0-5",
    feature = "backend-combined-wav-0-6"
))]
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Missing command line argument.");
    } else {
        let samplerate = 44100;
        let input_midi_filename = args[1].clone();
        println!("Reading midi input file.");
        let input_midi_data = fs::read(input_midi_filename).unwrap();
        println!("Parsing midi input file.");
        let smf = Smf::parse(&input_midi_data).unwrap();

        todo!();
    }
}

#[cfg(not(all(
    feature = "backend-combined-midly-0-5",
    feature = "backend-combined-wav-0-6"
)))]
fn main() {
    println!("This example was compiled without support for midly and wav.");
    println!("Compile with passing `--backend-combined-midly-0-5,backend-combined-wav-0-6`");
    println!("as parameter to `cargo`.");
}
