//! Defines the different backends.
#[cfg(feature = "jack-backend")]
pub mod jack_backend;
#[cfg(feature = "vst-backend")]
pub mod vst_backend;

/// Defines an interface for communicating with the host or server of the backend,
/// e.g. the VST host when using VST or the  Jack server when using Jack.
pub trait HostInterface {
    /// Return whether the output buffers are zero-initialized.
    /// Returns `false` when in doubt.
    fn output_initialized(&self) -> bool;
}
