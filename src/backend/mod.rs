//! Backends.
//!
//! Pre-defined backends
//! ====================
//! `rsynth` currently supports the following back-ends:
//! * [`combined`] combine different back-ends for audio input, audio output, midi input and
//!     midi output, mostly for offline rendering and testing (behind various features)
//! * [`jack`] (behind the `backend-jack` feature)
//! * [`vst`] (behind the `backend-vst` feature)
//!
//! These backends are currently in the `rsynth` crate, but we may eventually move them to
//! separate crates.
//!
//! Custom backends
//! ===============
//! You can write a backend in a separate crate. If you encounter problems that prevent you
//! from writing your backend in a separate crate (e.g., we have forgotten to
//! mark something as `pub`), let us know by opening an issue.
//!
//! Publishing a backend crate
//! --------------------------
//!
//! When you publish a backend crate, let us know by opening an issue or pull request
//! so that we can link to it in the documentation of rsynth.
//!
//! [`jack`]: ./jack_backend/index.html
//! [`vst`]: ./vst_backend/index.html
//! [`combined`]: ./combined/index.html
#[cfg(feature = "backend-combined")]
pub mod combined;
#[cfg(feature = "backend-jack")]
pub mod jack_backend;
#[cfg(feature = "backend-vst")]
pub mod vst_backend;

/// Defines an interface for communicating with the host or server of the backend,
/// e.g. the VST host when using VST or the  Jack server when using Jack.
pub trait HostInterface {
    /// Return whether the output buffers are zero-initialized.
    /// Returns `false` when in doubt.
    ///
    /// # Example
    ///
    /// The following example illustrates how `output_initialized()` can be used in
    /// combination with the `set` method on `AudioBufferOut` to initialize the output
    /// buffers to zero in an implementation of the [`ContextualAudioRenderer`] trait.
    ///
    /// ```
    /// use rsynth::ContextualAudioRenderer;
    /// use rsynth::backend::HostInterface;
    /// use rsynth::buffer::AudioBufferInOut;
    /// struct MyPlugin { /* ... */ }
    /// impl<H> ContextualAudioRenderer<f32, H> for MyPlugin
    /// where H: HostInterface
    /// {
    ///     fn render_buffer(
    ///         &mut self,
    ///         buffer: &mut AudioBufferInOut<f32>,
    ///         context: &mut H)
    ///     {
    ///         if ! context.output_initialized() {
    ///             buffer.outputs().set(0.0);
    ///         }
    ///         // The rest of the audio rendering.
    ///     }
    /// }
    /// ```
    ///
    /// [`ContextualEventHandler`]: ../event/trait.ContextualEventHandler.html
    /// [`rsynth::utilities::zero_init`]: ../utilities/fn.initialize_to_zero.html
    fn output_initialized(&self) -> bool;

    /// Stop processing.
    /// For backends that do not support stopping, this is a no-op.
    /// For back-ends that do support stopping and that implement the `Stop` trait,
    /// this stops the processing.
    fn stop(&mut self) {}
}

/// A marker trait that indicates that the backend can be stopped.
///
/// # Example
/// The following illustrates a plugin that works with backends that support stopping.
/// Based on some condition (`plugin_has_finished`), the plugin can signal to the backend
/// that processing is finished by calling `stop()`.
///
/// ```
/// use rsynth::ContextualAudioRenderer;
/// use rsynth::backend::{HostInterface, Stop};
/// use rsynth::buffer::AudioBufferInOut;
/// struct MyPlugin { /* ... */ }
/// impl<H> ContextualAudioRenderer<f32, H> for MyPlugin
/// where H: HostInterface + Stop
/// {
///     fn render_buffer(
///         &mut self,
///         buffer: &mut AudioBufferInOut<f32>,
///         context: &mut H)
///     {
///         let plugin_has_finished = unimplemented!();
///         if plugin_has_finished {
///             context.stop();
///         }
///     }
/// }
/// ```
pub trait Stop: HostInterface {}
