//! Defines the JACK backend and the VST backend.
#[cfg(feature = "jack-backend")]
pub mod jack_backend;
pub mod vst_backend;

/// Defines an interface for communicating with the host or server.
pub trait HostInterface {}

pub trait WithHost<H: HostInterface> {
    fn host(&self) -> &H;
}

/*
pub trait WithHostMut {
    type Host: HostInterface;
    fn host_mut(&mut self) -> &mut Self::Host;
}
*/

impl<T, H: HostInterface> WithHost<H> for T where T: AsRef<H> {
    fn host(&self) -> &H {
        self.as_ref()
    }
}

/*
impl<T, U: WithHostMut> WithHostMut for T where T: AsMut<U>{
    type Host = U::Host;
    fn host_mut(&mut self) -> &mut Self::Host {
        self.as_mut().host_mut()
    }
}
*/
