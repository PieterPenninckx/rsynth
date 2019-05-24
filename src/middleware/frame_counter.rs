use asprim::AsPrim;
use num_traits::Float;
#[cfg(feature = "stable")]
use syllogism::IsNot;

use crate::Plugin;
use crate::backend::HostInterface;
use crate::event::EventHandler;

/// Example middleware to illustrate how middleware can interfere with the context.
pub struct FrameCounter {
    number_of_frames_rendered: usize
}

impl FrameCounter {
    pub fn number_of_frames_rendered(&self) -> usize {
        self.number_of_frames_rendered
    }
}

pub struct FrameCounterContext<'sc, 'cc, C> {
    frame_counter: &'sc mut FrameCounter,
    child_context: &'cc mut C
}

// TODO: improve this part.
impl<H: HostInterface> IsNot<H> for FrameCounter {}

impl<'sc, 'cc, C, T: IsNot<FrameCounter>> AsRef<T> for FrameCounterContext<'sc, 'cc, C>
where C: AsRef<T>
{
    fn as_ref(&self) -> &T {
        self.child_context.as_ref()
    }
}

impl<'sc, 'cc, C, T: IsNot<FrameCounter>> AsMut<T> for FrameCounterContext<'sc, 'cc, C>
where C: AsMut<T>
{
    fn as_mut(&mut self) -> &mut T {
        self.child_context.as_mut()
    }
}

impl<'sc, 'cc, C> AsRef<FrameCounter> for FrameCounterContext<'sc, 'cc, C> {
    fn as_ref(&self) -> &FrameCounter {
        &self.frame_counter
    }
}

impl<'sc, 'cc, C> AsMut<FrameCounter> for FrameCounterContext<'sc, 'cc, C> {
    fn as_mut(&mut self) -> &mut FrameCounter {
        &mut self.frame_counter
    }
}

pub struct FrameCounterMiddleware<P> {
    sample_counter: FrameCounter,
    child_plugin: P
}

pub trait WithFrameCounter {
    fn sample_counter(&self) -> &FrameCounter;
}
impl<T> WithFrameCounter for T where T: AsRef<FrameCounter> {
    fn sample_counter(&self) -> &FrameCounter {
        self.as_ref()
    }
}

pub trait WithFrameCounterMut {
    fn sample_counter_mut(&mut self) -> &mut FrameCounter;
}
impl<T> WithFrameCounterMut for T where T: AsMut<FrameCounter> {
    fn sample_counter_mut(&mut self) -> &mut FrameCounter {
        self.as_mut()
    }
}

impl<P, C> Plugin<C> for FrameCounterMiddleware<P>
where for<'sc, 'cc> P: Plugin<FrameCounterContext<'sc, 'cc, C>>
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

    fn render_buffer<F>(&mut self, inputs: &[&[F]], outputs: &mut[&mut [F]], context: &mut C) where
        F: Float + AsPrim {
        if outputs.len() > 0 {
            self.sample_counter.number_of_frames_rendered += outputs[0].len();
        }
        let mut new_context = FrameCounterContext {
            frame_counter: &mut self.sample_counter,
            child_context: context
        };
        self.child_plugin.render_buffer(inputs, outputs, &mut new_context);
    }
}

impl<E, P, C> EventHandler<E, C> for FrameCounterMiddleware<P>
where
    for<'sc, 'cc> P: EventHandler<E, FrameCounterContext<'sc, 'cc, C>>
{
    fn handle_event(&mut self, event: E, context: &mut C) {
        let mut new_context = FrameCounterContext {
            frame_counter: &mut self.sample_counter,
            child_context: context
        };
        self.child_plugin.handle_event(event, &mut new_context);
    }
}
