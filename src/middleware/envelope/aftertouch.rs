use crate::envelope::Envelope;
use crate::event::{EventHandler, RawMidiEvent, Timed};
pub struct AfterTouchMarker;
use super::EnvelopeContext;
use crate::dev_utilities::transparent::Transparent;
use core::marker::PhantomData;
use syllogism::Specialize;

pub struct AfterTouchContext<E> {
    envelope: E,
}

impl<E> EnvelopeContext for AfterTouchContext<E> {
    type Marker = AfterTouchMarker;
    type Data = E;
    fn data(&mut self) -> &mut Self::Data {
        &mut self.envelope
    }
}

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

impl<T> AfterTouchEvent for Timed<T>
where
    T: AfterTouchEvent,
{
    fn aftertouch(&self) -> Option<u8> {
        self.event.aftertouch()
    }
}

pub struct AfterTouchMiddleware<Event, E, C, T>
where
    for<'a> E: Envelope<'a, T>,
{
    envelope: E,
    child: C,
    _phantom_event: PhantomData<Event>,
    _phantom_t: PhantomData<T>,
}

impl<Event, E, C, T> Transparent for AfterTouchMiddleware<Event, E, C, T>
where
    for<'a> E: Envelope<'a, T>,
{
    type Inner = C;

    fn get(&self) -> &Self::Inner {
        &self.child
    }

    fn get_mut(&mut self) -> &mut Self::Inner {
        &mut self.child
    }
}

impl<Event: AfterTouchEvent, E, C, T> AfterTouchMiddleware<Event, E, C, T>
where
    for<'a> E: Envelope<'a, T>,
{
    fn new(child: C, envelope: E) -> Self {
        Self {
            envelope,
            child,
            _phantom_event: PhantomData,
            _phantom_t: PhantomData,
        }
    }
}

impl<Event, E, C, T> AfterTouchMiddleware<Event, E, C, T>
where
    Event: AfterTouchEvent,
    for<'a> E: Envelope<'a, T, EventType = Timed<u8>>,
{
    fn handle_aftertouch_event(&mut self, event: Timed<Event>) {
        if let Some(aftertouch) = event.event.aftertouch() {
            self.envelope.insert_event(Timed {
                time_in_frames: event.time_in_frames,
                event: aftertouch,
            })
        }
    }
}

#[cfg(not(feature = "stable"))]
impl<Event, E, C, T, GenericEvent, Context> EventHandler<GenericEvent, Context>
    for AfterTouchMiddleware<Event, E, C, T>
where
    Event: AfterTouchEvent,
    for<'a> E: Envelope<'a, T, EventType = Timed<u8>>,
    C: EventHandler<GenericEvent, Context>,
{
    default fn handle_event(&mut self, event: GenericEvent, context: &mut Context) {
        self.child.handle_event(event, context);
    }
}

#[cfg(not(feature = "stable"))]
impl<Event, E, C, T, Context> EventHandler<Timed<Event>, Context>
    for AfterTouchMiddleware<Event, E, C, T>
where
    Event: AfterTouchEvent,
    for<'a> E: Envelope<'a, T, EventType = Timed<u8>>,
    C: EventHandler<Timed<Event>, Context>,
{
    fn handle_event(&mut self, event: Timed<Event>, context: &mut Context) {
        self.handle_aftertouch_event(event);
        self.child.handle_event(event, context);
    }
}

#[cfg(feature = "stable")]
impl<Event, E, C, T, GenericEvent, Context> EventHandler<GenericEvent, Context>
    for AfterTouchMiddleware<Event, E, C, T>
where
    Event: Specialize<AfterTouchEvent>,
    for<'a> E: Envelope<'a, T, EventType = Timed<u8>>,
    C: EventHandler<GenericEvent, Context>,
{
    fn handle_event(&mut self, event: E, context: &mut C) {
        unimplemented!();
        self.child.handle_event(event, context);
    }
}
