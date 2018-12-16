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
use simplelog::*;

#[cfg(feature="jack-backend")]
use rsynth::middleware::polyphony::{Polyphonic, SimpleVoiceStealer};
#[cfg(feature="jack-backend")]
use rsynth::middleware::zero_init::ZeroInit;
#[cfg(feature="jack-backend")]
use rsynth::backend::output_mode::Additive;
#[cfg(feature="jack-backend")]
use rsynth::backend::jack_backend::run;

#[cfg(feature="jack-backend")]
fn main() {
    CombinedLogger::init(
        vec![TermLogger::new(LevelFilter::Warn, Config::default()).unwrap()]
    ).unwrap();
    let mut voices = Vec::new();
    for _ in 0 .. 6 {
        voices.push(Sound::<Additive>::default());
    }
    let polyphony = Polyphonic::new(SimpleVoiceStealer::new(), voices);
    let zero_initialized = ZeroInit::new(polyphony);
    run(zero_initialized);
}

#[cfg(not(feature="jack-backend"))]
fn main() {
    println!("This example was compiled without support for jack.");
    println!("Compile with passing `--features jack-backend`");
    println!("as parameter to `cargo`.");
}