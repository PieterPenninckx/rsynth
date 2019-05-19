//! Defines the JACK backend and the VST backend.
#[cfg(feature = "jack-backend")]
pub mod jack_backend;
pub mod vst_backend;

/// Defines an interface for communicating with the host or server.
pub trait HostInterface {}
