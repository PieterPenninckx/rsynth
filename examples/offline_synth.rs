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
extern crate asprim;
#[cfg(feature = "backend-combined-midly-0-5")]
extern crate midly_0_5;
extern crate num_traits;
extern crate rand;
extern crate rsynth;

mod example_synth;
use example_synth::*;

#[cfg(feature = "backend-combined")]
use rsynth::backend::combined::dummy::{AudioDummy, MidiDummy};
#[cfg(feature = "backend-combined-wav-0-6")]
use rsynth::backend::combined::memory::wav_0_6::{read, write, BitDepth, Header};
#[cfg(feature = "backend-combined")]
use rsynth::backend::combined::memory::AudioBufferWriter;
#[cfg(feature = "backend-combined-midly-0-5")]
use rsynth::backend::combined::midly::{midly_0_5::Smf, MidlyMidiReader};
#[cfg(feature = "backend-combined")]
use rsynth::backend::combined::run;
use rsynth::buffer::AudioChunk;
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
        let mut output_buffer = AudioChunk::<f32>::new(2);
        let audio_buffer_writer = AudioBufferWriter::new(&mut output_buffer);
        let mut plugin = NoisePlayer::new();
        let buffer_size_in_frames = 256; // Quite arbitrarily.

        let number_of_seconds = 2;
        let audio_in = AudioDummy::with_sample_rate_and_length(
            samplerate,
            number_of_seconds * samplerate as usize,
        );
        let midi_event_reader = MidlyMidiReader::new(&smf).unwrap();
        let midi_out = MidiDummy::new();
        println!("Rendering {} tracks of audio.", number_of_seconds);
        run(
            &mut plugin,
            buffer_size_in_frames,
            audio_in,
            audio_buffer_writer,
            midi_event_reader,
            midi_out,
        )
        .unwrap();

        // Now output_buffer contains the data.
        let output_data_interlaced = output_buffer
            .interlaced()
            .map(|s| (s * (i16::MAX as f32)) as i16)
            .collect();
        let header = Header::new(1, 2, samplerate, 16);
        let track = BitDepth::Sixteen(output_data_interlaced);

        println!("Opening output file.");
        let output_wav_filename = args[2].clone();
        let mut output_file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output_wav_filename)
            .unwrap();
        println!("Writing to output file.");
        // Note: normally you will probably want to use a buffered writer.
        write(header, &track, &mut output_file).unwrap();
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
