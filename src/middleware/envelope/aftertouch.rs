use super::EnvelopeContext;
use crate::envelope::Envelope;
use crate::event::{EventHandler, RawMidiEvent, Timed};
use crate::{dev_utilities::transparent::Transparent, Plugin};
use asprim::AsPrim;
use core::marker::PhantomData;
use num_traits::Float;
#[cfg(feature = "stable")]
use syllogism::{Distinction, Specialize};

pub struct AfterTouchContext<Envl> {
    envelope: Envl,
}

pub struct AfterTouchMarker;

impl<Envl> EnvelopeContext for AfterTouchContext<Envl> {
    type Marker = AfterTouchMarker;
    type Data = Envl;
    fn data(&mut self) -> &mut Self::Data {
        &mut self.envelope
    }
}

wrap_context!(EnvelopeContextWrapper, AfterTouchContext<E>, E);

pub trait AfterTouchEvent: Copy {
    fn aftertouch(&self) -> Option<u8>;
}

impl AfterTouchEvent for RawMidiEvent {
    fn aftertouch(&self) -> Option<u8> {
        let state_and_chanel = self.data()[0];

        if state_and_chanel & 0xF0 == 0xD0 {
            Some(self.data()[1])
        } else {
            None
        }
    }
}

pub struct AfterTouchMiddleware<Event, Envl, Child, T>
where
    for<'a> Envl: Envelope<'a, T>,
{
    envelope_context: AfterTouchContext<Envl>,
    child: Child,
    _phantom_event: PhantomData<Event>,
    _phantom_t: PhantomData<T>,
}

impl<Event, Envl, Child, T> Transparent for AfterTouchMiddleware<Event, Envl, Child, T>
where
    for<'a> Envl: Envelope<'a, T>,
{
    type Inner = Child;

    fn get(&self) -> &Self::Inner {
        &self.child
    }

    fn get_mut(&mut self) -> &mut Self::Inner {
        &mut self.child
    }
}

impl<Event: AfterTouchEvent, Envl, Child, T> AfterTouchMiddleware<Event, Envl, Child, T>
where
    for<'a> Envl: Envelope<'a, T>,
{
    fn new(child: Child, envelope: Envl) -> Self {
        Self {
            envelope_context: AfterTouchContext { envelope },
            child,
            _phantom_event: PhantomData,
            _phantom_t: PhantomData,
        }
    }
}

impl<Event, Envl, Child, T> AfterTouchMiddleware<Event, Envl, Child, T>
where
    Event: AfterTouchEvent,
    for<'a> Envl: Envelope<'a, T, EventType = Timed<u8>>,
{
    fn handle_aftertouch_event(&mut self, event: Timed<Event>) {
        if let Some(aftertouch) = event.event.aftertouch() {
            self.envelope_context.envelope.insert_event(Timed {
                time_in_frames: event.time_in_frames,
                event: aftertouch,
            })
        }
    }
}

#[cfg(not(feature = "stable"))]
impl<Event, Envl, Child, T, GenericEvent, Context> EventHandler<GenericEvent, Context>
    for AfterTouchMiddleware<Event, Envl, Child, T>
where
    Event: AfterTouchEvent,
    for<'a> Envl: Envelope<'a, T, EventType = Timed<u8>>,
    for<'ac, 'cc> Child:
        EventHandler<GenericEvent, EnvelopeContextWrapper<'ac, 'cc, Context, Envl>>,
{
    default fn handle_event(&mut self, event: GenericEvent, context: &mut Context) {
        let mut wrapped_context = EnvelopeContextWrapper::new(&mut self.envelope_context, context);
        self.child.handle_event(event, &mut wrapped_context);
    }
}

#[cfg(not(feature = "stable"))]
impl<Event, Envl, Child, T, Context> EventHandler<Timed<Event>, Context>
    for AfterTouchMiddleware<Event, Envl, Child, T>
where
    Event: AfterTouchEvent,
    for<'a> Envl: Envelope<'a, T, EventType = Timed<u8>>,
    for<'ac, 'cc> Child:
        EventHandler<Timed<Event>, EnvelopeContextWrapper<'ac, 'cc, Context, Envl>>,
{
    fn handle_event(&mut self, event: Timed<Event>, context: &mut Context) {
        self.handle_aftertouch_event(event);
        let mut wrapped_context = EnvelopeContextWrapper::new(&mut self.envelope_context, context);
        self.child.handle_event(event, &mut wrapped_context);
    }
}

#[cfg(feature = "stable")]
impl<Event, E, C, T, GenericEvent, Context> EventHandler<GenericEvent, Context>
    for AfterTouchMiddleware<Event, E, C, T>
where
    GenericEvent: Specialize<Timed<Event>>,
    Event: AfterTouchEvent,
    for<'a> E: Envelope<'a, T, EventType = Timed<u8>>,
    for<'ac, 'cc> C: EventHandler<GenericEvent, EnvelopeContextWrapper<'ac, 'cc, Context, E>>
        + EventHandler<Timed<Event>, EnvelopeContextWrapper<'ac, 'cc, Context, E>>,
{
    fn handle_event(&mut self, event: GenericEvent, context: &mut Context) {
        match event.specialize() {
            Distinction::Special(special) => {
                self.handle_aftertouch_event(special);
                let mut wrapped_context =
                    EnvelopeContextWrapper::new(&mut self.envelope_context, context);
                self.child.handle_event(special, &mut wrapped_context);
            }
            Distinction::Generic(generic) => {
                let mut wrapped_context =
                    EnvelopeContextWrapper::new(&mut self.envelope_context, context);
                self.child.handle_event(generic, &mut wrapped_context);
            }
        }
    }
}

impl<Event, Envl, Child, T, Context> Plugin<Context> for AfterTouchMiddleware<Event, Envl, Child, T>
where
    for<'a> Envl: Envelope<'a, T, EventType = Timed<u8>>,
    for<'ac, 'cc> Child: Plugin<EnvelopeContextWrapper<'ac, 'cc, Context, Envl>>,
{
    const NAME: &'static str = Child::NAME;
    const MAX_NUMBER_OF_AUDIO_INPUTS: usize = Child::MAX_NUMBER_OF_AUDIO_INPUTS;
    const MAX_NUMBER_OF_AUDIO_OUTPUTS: usize = Child::MAX_NUMBER_OF_AUDIO_OUTPUTS;

    fn audio_input_name(index: usize) -> String {
        Child::audio_input_name(index)
    }

    fn audio_output_name(index: usize) -> String {
        Child::audio_output_name(index)
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.child.set_sample_rate(sample_rate);
    }

    fn render_buffer<F>(&mut self, inputs: &[&[F]], outputs: &mut [&mut [F]], context: &mut Context)
    where
        F: Float + AsPrim,
    {
        let mut wrapped_context = EnvelopeContextWrapper::new(&mut self.envelope_context, context);
        self.child
            .render_buffer(inputs, outputs, &mut wrapped_context);
    }
}
