//! Defines the different backends.
#[cfg(feature = "jack-backend")]
pub mod jack_backend;
#[cfg(feature = "vst-backend")]
pub mod vst_backend;
use crate::context::TransparentContext;

/// Defines an interface for communicating with the host or server.
pub trait HostInterface {}

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
