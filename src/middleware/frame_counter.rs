use asprim::AsPrim;
use num_traits::Float;
#[cfg(feature = "stable")]
use syllogism::IsNot;
use std::borrow::{Borrow, BorrowMut};

use crate::Plugin;
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

#[cfg(feature = "stable")]
impl<'sc, 'cc, C> Borrow<FrameCounter> for FrameCounterContext<'sc, 'cc, C>
{
    fn borrow(&self) -> &FrameCounter {
        self.frame_counter
    }
}

#[cfg(feature = "stable")]
impl<'sc, 'cc, C, T> Borrow<T> for FrameCounterContext<'sc, 'cc, C>
where
    C: Borrow<T>,
    T: IsNot<FrameCounter> 
{
    fn borrow(&self) -> &T {
        (*self.child_context).borrow()
    }
}

#[cfg(feature = "stable")]
impl<'sc, 'cc, C> BorrowMut<FrameCounter> for FrameCounterContext<'sc, 'cc, C>
{
    fn borrow_mut(&mut self) -> &mut FrameCounter {
        self.frame_counter
    }
}

#[cfg(feature = "stable")]
impl<'sc, 'cc, C, T> BorrowMut<T> for FrameCounterContext<'sc, 'cc, C>
where
    C: BorrowMut<T>,
    T: IsNot<FrameCounter> 
{
    fn borrow_mut(&mut self) -> &mut T {
        self.child_context.borrow_mut()
    }
}

#[cfg(not(feature = "stable"))]
pub mod nightly {
    use super::{FrameCounter, FrameCounterContext};
    use std::borrow::{Borrow, BorrowMut};

    // Thanks to Lymia for this trick.
    // For more info, see
    // https://github.com/rust-lang/rust/issues/31844#issuecomment-397650553
    trait UniversalBorrow<T> {
        fn borrow(&self) -> &T;
    }

    trait UniversalBorrowMut<T> {
        fn borrow_mut(&mut self) -> &mut T;
    }

    impl<'sc, 'cc, C, T> UniversalBorrow<T> for FrameCounterContext<'sc, 'cc, C> {
        default fn borrow(&self) -> &T {
            unreachable!();
        }
    }

    impl<'sc, 'cc, C, T> UniversalBorrow<T> for FrameCounterContext<'sc, 'cc, C>
        where
            FrameCounterContext <'sc, 'cc, C>: UniversalBorrow<T> {
        fn borrow(&self) -> & T {
            self.borrow();
        }
    }

    impl<'sc, 'cc, C, T> UniversalBorrowMut<T> for FrameCounterContext<'sc, 'cc, C> {
        default fn borrow_mut(&mut self) -> &mut T {
            unreachable!();
        }
    }

    impl<'sc, 'cc, C, T> UniversalBorrowMut<T> for FrameCounterContext<'sc, 'cc, C>
        where
            FrameCounterContext <'sc, 'cc, C>: UniversalBorrow<T> {
        fn borrow_mut(&mut self) -> &mut T {
            self.borrow_mut();
        }
    }

    trait GenericOrSpecial<T> {}

    impl<'sc, 'cc, C, T> GenericOrSpecial<T> for FrameCounterContext<'sc, 'cc, C>
    where
        FrameCounterContext < 'sc, 'cc, C >: GenericBorrow<T> {
    }

    impl<'sc, 'cc, C, T> GenericOrSpecial<FrameCounter> for FrameCounterContext<'sc, 'cc, C> {
    }

    impl<'sc, 'cc, C, T> Borrow<T> for FrameCounterContext<'sc, 'cc, C>
    where
        FrameCounterContext<'sc, 'cc, C>: GenericOrSpecial<T> {
        fn borrow(&self) -> &T {
            <self as UniversalBorrow<T>>::borrow()
        }
    }

    impl<'sc, 'cc, C, T> Borrow<FrameCounter> for FrameCounterContext<'sc, 'cc, C>
        where
            FrameCounterContext<'sc, 'cc, C>: GenericOrSpecial<FrameCounter> {
        fn borrow(&self) -> &FrameCounter {
            self.frame_counter
        }
    }

    impl<'sc, 'cc, C, T> BorrowMut<T> for FrameCounterContext<'sc, 'cc, C>
        where
            FrameCounterContext<'sc, 'cc, C>: GenericOrSpecial<T> {
        fn borrow_mut(&mut self) -> &mut T {
            <self as UniversalBorrowMut<T>>::borrow_mut()
        }
    }

    impl<'sc, 'cc, C, T> BorrowMut<FrameCounter> for FrameCounterContext<'sc, 'cc, C>
        where
            FrameCounterContext<'sc, 'cc, C>: GenericOrSpecial<FrameCounter> {
        fn borrow(&mut self) -> &mut FrameCounter {
            self.frame_counter
        }
    }
}

pub trait WithFrameCounter {
    fn frame_counter(&self) -> &FrameCounter;
}

impl<T> WithFrameCounter for T where T: Borrow<FrameCounter> {
    fn frame_counter(&self) -> &FrameCounter {
        self.borrow()
    }
}

pub trait WithFrameCounterMut {
    fn frame_counter_mut(&mut self) -> &mut FrameCounter;
}

impl<T> WithFrameCounterMut for T where T: BorrowMut<FrameCounter> {
    fn frame_counter_mut(&mut self) -> &mut FrameCounter {
        self.borrow_mut()
    }
}

pub struct FrameCounterMiddleware<P> {
    sample_counter: FrameCounter,
    child_plugin: P
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
