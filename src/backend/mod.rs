//! Defines the different backends.
#[cfg(feature = "backend-file")]
pub mod file_backend;
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
    /// combination with `rsynth::utilities::zero_init` to initialize the output
    /// buffers to zero in an implementation of the `ContextualAudioRenderer` trait.
    ///
    /// ```
    /// use rsynth::ContextualAudioRenderer;
    /// # use rsynth::AudioRendererMeta;
    /// use rsynth::backend::HostInterface;
    /// use rsynth::utilities::initialize_to_zero;
    /// # struct MyPlugin {}
    /// # impl AudioRendererMeta for MyPlugin {
    /// #    const MAX_NUMBER_OF_AUDIO_INPUTS: usize = 0;
    /// #    const MAX_NUMBER_OF_AUDIO_OUTPUTS: usize = 2;
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
    fn output_initialized(&self) -> bool;
}
