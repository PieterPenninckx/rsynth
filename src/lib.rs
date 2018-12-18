//! # Rsynth
//! The `rsynth` crate makes it easier to write software synthesizers.
//!
//! # The `Plugin` trait
//! The functionality of the plugin that is common to all back-ends is defined
//! by the [`Plugin`] trait.
//!
//! # Back-ends
//! `rsynth` currently supports two back-ends:
//!
//! * [`jack`]
//! * [`vst`]
//!
//! In order to support a specific back-end, plugins may additionally need
//! to implement a backend-specific trait on top of the `Plugin` trait. See the
//! documentation of each back-end for more information.
//!
//! # Middleware
//! You can add features, such as polyphony, to your plug-in by using middleware.
//! Typically, suppose `M` is middleware and your plugin `P` implement the `Plugin` trait and
//! any other backend-specific trait, then `M<P>` also implements the `Plugin` trait
//! and the backend-specific traits `P` implements.
//! Currently, supported middleware is
//!
//! * [`Polyphony`]
//! * [`ZeroInit`]
//!
//! [`Plugin`]: ./backend/trait.Plugin.html
//! [`jack`]: ./backend/jack_backend/index.html
//! [`vst`]: ./backend/vst_backend/index.html
//! [`Polyphony`]: ./middleware/polyphony/index.html
//! [`ZeroInit`]: ./middleware/zero_init/index.html
#[macro_use]
extern crate log;
extern crate asprim;
extern crate core;
#[cfg(feature = "jack-backend")]
extern crate jack;
extern crate num;
extern crate num_traits;
extern crate vst;

pub mod backend;
pub mod dsp;
pub mod envelope;
pub mod middleware;
pub mod note;
pub mod point;
pub mod synth;
