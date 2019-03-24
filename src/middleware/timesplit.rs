// Currently largely unimplemented because this is only to check if the
// eventing system can be used in practice.

use asprim::AsPrim;
use backend::{Plugin, Transparent};
use backend::event::{Timed, WithTime};
use num_traits::Float;
use backend::event::EventHandler;
use is_not::IsNot;
use downcast::{DowncastCheck, Downcast, DowncastRef};

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

impl<P, E> Plugin for TimeSplit<P, E>
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
        unimplemented!()
    }
}

impl<P, E, EE> EventHandler<EE> for TimeSplit<P, E>
where 
    P: EventHandler<EE>,
    EE: DowncastCheck<E> + Downcast<E>,
    EE: WithTime
{
    fn handle_event(&mut self, event: EE) {
        if <EE as DowncastCheck<E>>::can_downcast(&event) {
            if let Some(time) = event.time_in_frames() {
                if time != 0 {
                    if self.buffer.len() < self.buffer.capacity() {
                        if let Some(e) = event.downcast() {
                            self.buffer.push(Timed{time_in_frames: time, event: e});
                            return;
                        } else {
                            unimplemented!()
                        }
                    } else {
                        unimplemented!()
                    }
                }
            }
        }
        self.plugin.handle_event(event);
    }
}
