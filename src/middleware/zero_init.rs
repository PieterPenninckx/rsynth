//! Middleware for initializing the output to zero before actually calling the plugin.
//!
//! You will need this when you use a back-end that does not zero-initialize the output buffer
//! before calling the plugin and you are using the `Polyphony` middleware.

use asprim::AsPrim;
use backend::{Plugin, Transparent};
use num_traits::Float;
use backend::event::EventHandler;

/// Set all output values to 0 before calling `render_buffer` on the "child".
pub struct ZeroInit<P> {
    plugin: P,
}

impl<P> ZeroInit<P> {
    /// Create a new `ZeroInit` with the given "child plugin".
    pub fn new(plugin: P) -> Self {
        Self { plugin }
    }
}

impl<P> Transparent for ZeroInit<P> {
    type Inner = P;

    fn get(&self) -> &P {
        &self.plugin
    }

    fn get_mut(&mut self) -> &mut P {
        &mut self.plugin
    }
}

impl<P> Plugin for ZeroInit<P>
where
    P: Plugin,
{
    const NAME: &'static str = P::NAME;
    const MAX_NUMBER_OF_AUDIO_INPUTS: usize = P::MAX_NUMBER_OF_AUDIO_INPUTS;
    const MAX_NUMBER_OF_AUDIO_OUTPUTS: usize = P::MAX_NUMBER_OF_AUDIO_OUTPUTS;

    fn audio_input_name(index: usize) -> String {
        P::audio_input_name(index)
    }

    fn audio_output_name(index: usize) -> String {
        P::audio_output_name(index)
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.plugin.set_sample_rate(sample_rate);
    }

    fn render_buffer<F>(&mut self, inputs: &[&[F]], outputs: &mut [&mut [F]])
        where
            F: Float + AsPrim,
    {
        for output in outputs.iter_mut() {
            for sample in output.iter_mut() {
                *sample = F::zero();
            }
        }
        self.plugin.render_buffer(inputs, outputs);
    }
}

impl<E, P> EventHandler<E> for ZeroInit<P>
where P: EventHandler<E>
{
    fn handle_event(&mut self, event: E) {
        self.plugin.handle_event(event);
    }
}
