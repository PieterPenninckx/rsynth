use crate::event::{RawMidiEvent, Timed};
pub struct AfterTouchMarker;
use core::marker::PhantomData;

pub struct AfterTouchContext<E> {
    envelope: E,
}

pub trait EnvelopeContext {
    type Marker;
    type Data;
    fn data(&mut self) -> &mut Self::Data;
}

impl<E> EnvelopeContext for AfterTouchContext<E> {
    type Marker = AfterTouchMarker;
    type Data = E;
    fn data(&mut self) -> &mut Self::Data {
        &mut self.envelope
    }
}

pub trait AfterTouchEvent {
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

pub struct AfterTouchMiddleware<Event: AfterTouchEvent, E, C> {
    phantom: PhantomData<Event>,
    envelope: E,
    child: C,
}

impl<Event: AfterTouchEvent, E, C> AfterTouchMiddleware<Event, E, C> {
    fn new(child: C, envelope: E) -> Self {
        Self {
            phantom: PhantomData,
            envelope,
            child,
        }
    }
}

impl<Event: AfterTouchEvent, E, C> AfterTouchMiddleware<Event, E, C> {
    fn handle_aftertouch_event(event: Event) {
        if let Some(aftertouch) = event.aftertouch() {
            unimplemented!();
        }
    }
}
