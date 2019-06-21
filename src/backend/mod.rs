//! Defines the different backends.
#[cfg(feature = "jack-backend")]
pub mod jack_backend;
#[cfg(feature = "vst-backend")]
pub mod vst_backend;
use crate::context::TransparentContext;

/// Defines an interface for communicating with the host or server of the backend,
/// e.g. the VST host when using VST or the  Jack server when using Jack.
pub trait HostInterface {}

/// Interface that can be used to access the host from a context that supports it.
pub trait WithHost<H: HostInterface> {
    fn host(&mut self) -> &H;
}

impl<T, H: HostInterface> WithHost<H> for T
where
    T: TransparentContext<H>,
{
    fn host(&mut self) -> &H {
        self.get()
    }
}
