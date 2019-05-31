// Currently largely unimplemented because this is only to check if the
// eventing system can be used in practice.

use asprim::AsPrim;
use crate::Plugin;
use crate::event::{EventHandler, Timed};
use num_traits::Float;
use crate::dev_utilities::{transparent::Transparent};
#[cfg(feature = "stable")]
use syllogism::{Specialize, Distinction};

pub struct TimeSplit<P, E> {
    plugin: P,
    buffer: Vec<Timed<E>>
}

impl<P, E> TimeSplit<P, E> {
    pub fn new(plugin: P, capacity: usize) -> Self {
        Self { 
            plugin,
            buffer: Vec::with_capacity(capacity)
        }
    }

    fn save_event(&mut self, event: Timed<E>) {
        if event.time_in_frames != 0 {
            if self.buffer.len() < self.buffer.capacity() {
                self.buffer.push(event);
                return;
            } else {
                unimplemented!()
            }
        }
    }
}

impl<P, E> Transparent for TimeSplit<P, E> {
    type Inner = P;

    fn get(&self) -> &P {
        &self.plugin
    }

    fn get_mut(&mut self) -> &mut P {
        &mut self.plugin
    }
}

impl<P, E, C> Plugin<C> for TimeSplit<P, E>
where
    P: Plugin<C>,
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

    fn render_buffer<F>(&mut self, inputs: &[&[F]], outputs: &mut [&mut [F]], context: &mut C)
        where
            F: Float + AsPrim,
    {
        unimplemented!()
    }
}

#[cfg(feature = "stable")]
impl<P, E, EE, C> EventHandler<EE, C> for TimeSplit<P, E>
where 
    P: EventHandler<EE, C>,
    EE: Specialize<Timed<E>>
{
    fn handle_event(&mut self, event: EE, context: &mut C) {
        match <EE as Specialize<Timed<E>>>::specialize(event) {
            Distinction::Special(event) => {
                self.save_event(event);
            },
            Distinction::Generic(g) => {
                self.plugin.handle_event(g, context);
            }
        }
    }
}

#[cfg(not(feature = "stable"))]
impl<P, E, EE, C> EventHandler<EE, C> for TimeSplit<P, E>
    where
        P: EventHandler<EE, C>
{
    default fn handle_event(&mut self, event: EE, context: &mut C) {
        self.plugin.handle_event(event, context);
    }
}

#[cfg(not(feature = "stable"))]
impl<P, E, C> EventHandler<Timed<E>, C> for TimeSplit<P, E>
    where
        P: EventHandler<Timed<E>, C>
{
    fn handle_event(&mut self, event: Timed<E>, _context: &mut C) {
        self.save_event(event);
    }
}
