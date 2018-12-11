#[macro_use]
extern crate log;
extern crate simplelog;
extern crate asprim;
extern crate num_traits;
extern crate rand;
extern crate rsynth;

mod test_synth;
#[cfg(feature="jack-backend")]
use test_synth::*;


use rsynth::polyphony::{Polyphonic, SimpleVoiceStealer};
#[cfg(feature="jack-backend")]
use rsynth::backend::jack_backend::run;

#[cfg(feature="jack-backend")]
fn main() {
    let plugin = Sound::default();
    run(plugin);
}

#[cfg(not(feature="jack-backend"))]
fn main() {
    println!("This example was compiled without support for jack.");
    println!("Compile with passing `--features jack-backend`");
    println!("as parameter to `cargo`.");
}