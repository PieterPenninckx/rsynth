use asprim::AsPrim;
use num_traits::Float;
#[cfg(feature = "stable")]
use syllogism::IsNot;

use crate::context::{TransparentContext, TransparentContextMut};
use crate::event::EventHandler;
use crate::Plugin;

/// Example middleware to illustrate how middleware can interfere with the context.
pub struct FrameCounter {
    number_of_frames_rendered: usize,
}

impl FrameCounter {
    pub fn number_of_frames_rendered(&self) -> usize {
        self.number_of_frames_rendered
    }
}

pub struct FrameCounterContext<'sc, 'cc, C> {
    frame_counter: &'sc mut FrameCounter,
    child_context: &'cc mut C,
}

// Note: we cannot write a more generic implementation in the following style:
// pub struct GenericContextWrapper<'e, 'c, E, C> {
//      extra: &'e mut E,
//      child_context: &'c mut C,
// }
// because the compiler doesn't know that `E` does not implement `IsNot<E>`,
// so we would get into trouble with specialization.

#[cfg(feature = "stable")]
impl<'sc, 'cc, C> TransparentContext<FrameCounter> for FrameCounterContext<'sc, 'cc, C> {
    fn get(&self) -> &FrameCounter {
        self.frame_counter
    }
}

#[cfg(feature = "stable")]
impl<'sc, 'cc, C, T> TransparentContext<T> for FrameCounterContext<'sc, 'cc, C>
where
    C: TransparentContext<T>,
    T: IsNot<FrameCounter>,
{
    fn get(&self) -> &T {
        (*self.child_context).get()
    }
}

#[cfg(feature = "stable")]
impl<'sc, 'cc, C> TransparentContextMut<FrameCounter> for FrameCounterContext<'sc, 'cc, C> {
    fn get_mut(&mut self) -> &mut FrameCounter {
        self.frame_counter
    }
}

#[cfg(feature = "stable")]
impl<'sc, 'cc, C, T> TransparentContextMut<T> for FrameCounterContext<'sc, 'cc, C>
where
    C: TransparentContextMut<T>,
    T: IsNot<FrameCounter>,
{
    fn get_mut(&mut self) -> &mut T {
        self.child_context.get_mut()
    }
}

#[cfg(not(feature = "stable"))]
pub mod nightly {
    use super::{FrameCounter, FrameCounterContext};
    use crate::context::{TransparentContext, TransparentContextMut};

    // Thanks to Lymia for this trick.
    // For more info, see
    // https://github.com/rust-lang/rust/issues/31844#issuecomment-397650553
    trait UniversalTransparentContext<T> {
        fn get(&self) -> &T;
    }

    trait UniversalTransparentContextMut<T> {
        fn get_mut(&mut self) -> &mut T;
    }

    impl<'sc, 'cc, C, T> UniversalTransparentContext<T> for FrameCounterContext<'sc, 'cc, C> {
        default fn get(&self) -> &T {
            unreachable!();
        }
    }

    impl<'sc, 'cc, C, T> UniversalTransparentContext<T> for FrameCounterContext<'sc, 'cc, C>
    where
        C: TransparentContext<T>,
    {
        fn get(&self) -> &T {
            self.child_context.get()
        }
    }

    impl<'sc, 'cc, C, T> UniversalTransparentContextMut<T> for FrameCounterContext<'sc, 'cc, C> {
        default fn get_mut(&mut self) -> &mut T {
            unreachable!();
        }
    }

    impl<'sc, 'cc, C, T> UniversalTransparentContextMut<T> for FrameCounterContext<'sc, 'cc, C>
    where
        C: TransparentContextMut<T>,
    {
        fn get_mut(&mut self) -> &mut T {
            self.child_context.get_mut()
        }
    }

    trait GenericOrSpecial<T> {}

    impl<'sc, 'cc, C, T> GenericOrSpecial<T> for FrameCounterContext<'sc, 'cc, C>
    where
        C: TransparentContext<T>
    {
    }

    impl<'sc, 'cc, C> GenericOrSpecial<FrameCounter> for FrameCounterContext<'sc, 'cc, C> {}

    impl<'sc, 'cc, C, T> TransparentContext<T> for FrameCounterContext<'sc, 'cc, C>
    where
        FrameCounterContext<'sc, 'cc, C>: GenericOrSpecial<T>,
    {
        default fn get(&self) -> &T {
            <Self as UniversalTransparentContext<T>>::get(self)
        }
    }

    impl<'sc, 'cc, C> TransparentContext<FrameCounter> for FrameCounterContext<'sc, 'cc, C>
    where
        FrameCounterContext<'sc, 'cc, C>: GenericOrSpecial<FrameCounter>,
    {
        fn get(&self) -> &FrameCounter {
            self.frame_counter
        }
    }

    impl<'sc, 'cc, C, T> TransparentContextMut<T> for FrameCounterContext<'sc, 'cc, C>
    where
        FrameCounterContext<'sc, 'cc, C>: GenericOrSpecial<T>,
    {
        default fn get_mut(&mut self) -> &mut T {
            <Self as UniversalTransparentContextMut<T>>::get_mut(self)
        }
    }

    impl<'sc, 'cc, C> TransparentContextMut<FrameCounter> for FrameCounterContext<'sc, 'cc, C>
    where
        FrameCounterContext<'sc, 'cc, C>: GenericOrSpecial<FrameCounter>,
    {
        fn get_mut(&mut self) -> &mut FrameCounter {
            self.frame_counter
        }
    }
}

pub trait WithFrameCounter {
    fn frame_counter(&self) -> &FrameCounter;
}

impl<T> WithFrameCounter for T
where
    T: TransparentContext<FrameCounter>,
{
    fn frame_counter(&self) -> &FrameCounter {
        self.get()
    }
}

pub trait WithFrameCounterMut {
    fn frame_counter_mut(&mut self) -> &mut FrameCounter;
}

impl<T> WithFrameCounterMut for T
where
    T: TransparentContextMut<FrameCounter>,
{
    fn frame_counter_mut(&mut self) -> &mut FrameCounter {
        self.get_mut()
    }
}

pub struct FrameCounterMiddleware<P> {
    sample_counter: FrameCounter,
    child_plugin: P,
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
        if outputs.len() > 0 {
            self.sample_counter.number_of_frames_rendered += outputs[0].len();
        }
        let mut new_context = FrameCounterContext {
            frame_counter: &mut self.sample_counter,
            child_context: context,
        };
        self.child_plugin
            .render_buffer(inputs, outputs, &mut new_context);
    }
}

impl<E, P, C> EventHandler<E, C> for FrameCounterMiddleware<P>
where
    for<'sc, 'cc> P: EventHandler<E, FrameCounterContext<'sc, 'cc, C>>,
{
    fn handle_event(&mut self, event: E, context: &mut C) {
        let mut new_context = FrameCounterContext {
            frame_counter: &mut self.sample_counter,
            child_context: context,
        };
        self.child_plugin.handle_event(event, &mut new_context);
    }
}
