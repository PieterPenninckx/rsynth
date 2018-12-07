//! # Rsynth
//! The `rsynth` crate makes it easier to write software synthesizers.
//!
//! # Back-ends
//! `rsynth` currently supports two back-ends:
//! 
//! * jack
//! * vst
//! 
//! The functionality of the plugin that is common to all back-ends is defined
//! by the `Plugin` trait. In order to support a specific back-end, the plugin
//! may additionally need to implement a backend-specific interface. See the
//! documentation of each back-end for more information.
//!
//! # Middleware
//! You can add features, such as polyphony, to your plug-in by using middleware.
//! Typically, suppose `M` is middleware and your plugin `P` implement the `Plugin` trait and
//! any other backend-specific trait, then `M<P>` also implements the `Plugin` trait
//! and the backend-pecific traits `P` implements.
//! Currently, supported middle ware is
//!
//! * polyphony
#[macro_use]
extern crate log;
extern crate asprim;
extern crate num;
extern crate num_traits;
extern crate vst;
#[cfg(feature="jack-backend")]
extern crate jack;
extern crate core;


pub mod dsp;
pub mod envelope;
pub mod note;
pub mod point;
pub mod synth;
pub mod polyphony;
pub mod backend;
