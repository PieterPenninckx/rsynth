use asprim::AsPrim;
use num_traits::Float;

use backend::Plugin;
use backend::HostInterface;
use backend::IsNot;

/// Example middleware to illustrate how middleware can interfere with the context.
// TODO: Rename to FrameCounter
pub struct SampleCounter {
    number_of_samples_rendered: usize
}

impl SampleCounter {
    fn number_of_samples_rendered(&self) -> usize {
        self.number_of_samples_rendered
    }
}

pub struct SampleCounterContext<'sc, 'cc, C> {
    sample_counter: &'sc mut SampleCounter,
    child_context: &'cc mut C
}

impl<H: HostInterface> IsNot<H> for SampleCounter {}

impl<'sc, 'cc, C, T: IsNot<SampleCounter>> AsRef<T> for SampleCounterContext<'sc, 'cc, C> 
where C: AsRef<T>
{
    fn as_ref(&self) -> &T {
        self.child_context.as_ref()
    }
}

impl<'sc, 'cc, C, T: IsNot<SampleCounter>> AsMut<T> for SampleCounterContext<'sc, 'cc, C> 
where C: AsMut<T>
{
    fn as_mut(&mut self) -> &mut T {
        self.child_context.as_mut()
    }
}

impl<'sc, 'cc, C> AsRef<SampleCounter> for SampleCounterContext<'sc, 'cc, C> {
    fn as_ref(&self) -> &SampleCounter {
        &self.sample_counter
    }
}

impl<'sc, 'cc, C> AsMut<SampleCounter> for SampleCounterContext<'sc, 'cc, C> {
    fn as_mut(&mut self) -> &mut SampleCounter {
        &mut self.sample_counter
    }
}

pub struct SampleCounterMiddleware<P> {
    sample_counter: SampleCounter,
    child_plugin: P
}

pub trait WithSampleCounter {
    fn sample_counter(&self) -> &SampleCounter;
}
impl<T> WithSampleCounter for T where T: AsRef<SampleCounter> {
    fn sample_counter(&self) -> &SampleCounter {
        self.as_ref()
    }
}

pub trait WithSampleCounterMut {
    fn sample_counter_mut(&mut self) -> &mut SampleCounter;
}
impl<T> WithSampleCounterMut for T where T: AsMut<SampleCounter> {
    fn sample_counter_mut(&mut self) -> &mut SampleCounter {
        self.as_mut()
    }
}

impl<P, E, C> Plugin<E, C> for SampleCounterMiddleware<P>
where for<'sc, 'cc> P: Plugin<E, SampleCounterContext<'sc, 'cc, C>>
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
            self.sample_counter.number_of_samples_rendered += outputs[0].len();
        }
        let mut new_context = SampleCounterContext {
            sample_counter: &mut self.sample_counter,
            child_context: context
        };
        self.child_plugin.render_buffer(inputs, outputs, &mut new_context);
    }

    fn handle_event(&mut self, event: &E, context: &mut C) {
        let mut new_context = SampleCounterContext {
            sample_counter: &mut self.sample_counter,
            child_context: context
        };
        self.child_plugin.handle_event(event, &mut new_context);
    }
}
