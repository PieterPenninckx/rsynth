//! Backends.
//!
//! Pre-defined backends
//! ====================
//! `rsynth` currently supports the following back-ends:
//! * [`combined`] reading and writing audio and midi files, or working in-memory (behind various features)
//! * [`jack`] (behind the `backend-jack` feature)
//! * [`vst`] (behind the backend-vst)
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
//! [`vst`]: ./bvst_backend/index.html
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
    /// combination with [`rsynth::utilities::zero_init`] to initialize the output
    /// buffers to zero in an implementation of the [`ContextualAudioRenderer`] trait.
    ///
    /// ```
    /// use rsynth::ContextualAudioRenderer;
    /// # use rsynth::{AudioHandlerMeta, AudioHandler};
    /// use rsynth::backend::HostInterface;
    /// use rsynth::utilities::initialize_to_zero;
    /// # struct MyPlugin {}
    /// # impl AudioHandlerMeta for MyPlugin {
    /// #    fn max_number_of_audio_inputs(&self) -> usize { 0 }
    /// #    fn max_number_of_audio_outputs(&self) -> usize { 2 }
    /// # }
    /// # impl AudioHandler for MyPlugin {
    /// #    fn set_sample_rate(&mut self,sample_rate: f64) {
    /// #    }
    /// # }
    /// impl<H> ContextualAudioRenderer<f32, H> for MyPlugin
    /// where H: HostInterface
    /// {
    ///     fn render_buffer(
    ///         &mut self,
    ///         inputs: &[&[f32]],
    ///         outputs: &mut [&mut [f32]],
    ///         context: &mut H)
    ///     {
    ///         if ! context.output_initialized() {
    ///             initialize_to_zero(outputs);
    ///             // The rest of the audio rendering.
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// [`ContextualEventHandler`]: ../event/trait.ContextualEventHandler.html
    /// [`rsynth::utilities::zero_init`]: ../utilities/fn.initialize_to_zero.html
    fn output_initialized(&self) -> bool;
}
