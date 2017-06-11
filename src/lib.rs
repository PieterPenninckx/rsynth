#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
extern crate vst2;
extern crate asprim;
extern crate num_traits;
extern crate num;

pub mod voice;
pub mod synthesizer;
pub mod utility;