use crate::envelope::Envelope;
use crate::event::{RawMidiEvent, Timed};
pub struct AfterTouchMarker;
use super::EnvelopeContext;
use crate::dev_utilities::transparent::Transparent;
use core::marker::PhantomData;

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

pub struct AfterTouchMiddleware<Event, E, C> {
    phantom: PhantomData<Event>,
    envelope: E,
    child: C,
}

impl<Event, E, C> Transparent for AfterTouchMiddleware<Event, E, C> {
    type Inner = C;

    fn get(&self) -> &Self::Inner {
        &self.child
    }

    fn get_mut(&mut self) -> &mut Self::Inner {
        &mut self.child
    }
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

impl<Event, E, C> AfterTouchMiddleware<Event, E, C>
where
    Event: AfterTouchEvent,
    for<'a> E: Envelope<'a, (), EventType = Timed<u8>>,
    // TODO:                ^^ This should be a real type, now it's a dummy.
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
