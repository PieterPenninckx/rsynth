extern crate asprim;
extern crate num;
extern crate num_traits;
extern crate vst;
#[cfg(feature="jack-backend")]
extern crate jack;

pub mod dsp;
pub mod envelope;
pub mod note;
pub mod point;
pub mod synth;
pub mod voice;
pub mod backend;
