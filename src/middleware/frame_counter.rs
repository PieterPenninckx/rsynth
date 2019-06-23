use asprim::AsPrim;
use num_traits::Float;
#[cfg(feature = "stable")]
use syllogism::IsNot;

use crate::context::TransparentContext;
use crate::event::EventHandler;
use crate::Plugin;

/// Example middleware to illustrate how middleware can interfere with the context.
pub struct FrameCounter {
    number_of_frames_rendered: usize,
}

impl FrameCounter {
    fn new() -> Self {
        Self {
            number_of_frames_rendered: 0,
        }
    }
    pub fn number_of_frames_rendered(&self) -> usize {
        self.number_of_frames_rendered
    }
}

wrap_context!(FrameCounter, FrameCounterContext);

pub trait WithFrameCounter {
    fn frame_counter(&mut self) -> &mut FrameCounter;
}

impl<T> WithFrameCounter for T
where
    T: TransparentContext<FrameCounter>,
{
    fn frame_counter(&mut self) -> &mut FrameCounter {
        self.get()
    }
}

pub struct FrameCounterMiddleware<P> {
    sample_counter: FrameCounter,
    child_plugin: P,
}

impl<P> FrameCounterMiddleware<P> {
    pub fn new(child_plugin: P) -> Self {
        Self {
            sample_counter: FrameCounter::new(),
            child_plugin,
        }
    }
}

impl<P, C> Plugin<C> for FrameCounterMiddleware<P>
where
    for<'sc, 'cc> P: Plugin<FrameCounterContext<'sc, 'cc, C>>,
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
        self.child_plugin.set_sample_rate(sample_rate)
    }

    fn render_buffer<F>(&mut self, inputs: &[&[F]], outputs: &mut [&mut [F]], context: &mut C)
    where
        F: Float + AsPrim,
    {
        let mut new_context = FrameCounterContext::new(&mut self.sample_counter, context);
        self.child_plugin
            .render_buffer(inputs, outputs, &mut new_context);
        if outputs.len() > 0 {
            self.sample_counter.number_of_frames_rendered += outputs[0].len();
        }
    }
}

impl<E, P, C> EventHandler<E, C> for FrameCounterMiddleware<P>
where
    for<'sc, 'cc> P: EventHandler<E, FrameCounterContext<'sc, 'cc, C>>,
{
    fn handle_event(&mut self, event: E, context: &mut C) {
        let mut new_context = FrameCounterContext::new(&mut self.sample_counter, context);
        self.child_plugin.handle_event(event, &mut new_context);
    }
}

#[cfg(test)]
mod tests {
    use super::{FrameCounterMiddleware, WithFrameCounter};
    use crate::Plugin;
    use asprim::AsPrim;
    use num_traits::Float;

    struct PluginMock {
        index: usize,
        expected_frame_counter: Vec<usize>,
    }

    impl<C: WithFrameCounter> Plugin<C> for PluginMock {
        const NAME: &'static str = "";
        const MAX_NUMBER_OF_AUDIO_INPUTS: usize = 0;
        const MAX_NUMBER_OF_AUDIO_OUTPUTS: usize = 0;

        fn audio_input_name(index: usize) -> String {
            unimplemented!()
        }
        fn audio_output_name(index: usize) -> String {
            unimplemented!()
        }
        fn set_sample_rate(&mut self, sample_rate: f64) {}
        fn render_buffer<F>(&mut self, inputs: &[&[F]], outputs: &mut [&mut [F]], context: &mut C)
        where
            F: Float + AsPrim,
        {
            assert_eq!(
                context.frame_counter().number_of_frames_rendered(),
                self.expected_frame_counter[self.index]
            );
            self.index += 1;
        }
    }

    #[test]
    fn test_frame_counter() {
        let plugin_mock = PluginMock {
            index: 0,
            expected_frame_counter: vec![0, 1, 3, 103],
        };
        let mut p = FrameCounterMiddleware::new(plugin_mock);
        let mut out = vec![0.0_f32; 100];
        p.render_buffer(&[&[]], &mut [&mut out[0..1]], &mut ());
        p.render_buffer(&[&[]], &mut [&mut out[0..2]], &mut ());
        p.render_buffer(&[&[]], &mut [&mut out[0..100]], &mut ());
    }
}
