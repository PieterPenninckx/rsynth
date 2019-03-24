use is_not::IsNot;
use downcast::{Downcast, DowncastRef, DowncastMut, DowncastCheck};
// I see two options to allow handling a large amount of event types:
// * Work with an `EventHandler<E: EventType>` trait.
//   In order to enable middleware to both impl `EventHandler<MySpecificType>` and also
//   `impl<OtherType>` `EventHandler<OtherType>` (by delegating to a child) without getting
//   overlapping impl, we either need specialization, but this has not landed yet,
//   or we need to use the `IsNot` trick, which I am not too keen on.
// * Have one `handle_event` method that takes an `&dyn Any` parameter. This works for all event
//   types that are `'static` because of the `'static` lifetime requirement on `downcast_ref`.
//   But SysEx events are not `'static`. There are some possible work arounds for this:
//    * Store a pre-allocated `Vec<u8>` instead. This is what this struct does, but it has the
//      downside of not being "zero-copy".
//    * Use some unsafe blocks to get around the `'static` lifetime requirement on `downcast_ref`.
//      This would probably be tricky and also not completely memory safe.
//    * Replace the `&dyn Any` parameter by an `&DynamicEvent` parameter, where 
//      `enum DynamicEvent<'a> {AnyEvent(&'a dyn Any), SysexEvent(&'a[u8])}`. I tried this, but
//      I got lifetime errors while trying to `fn downcast(&'a self) -> Option<&SysExEvent<'a>>`.
//      I guess it's not really possible to create a lifetime "out of thin air" because
//      the `trait DownCastableTo<T> {fn downcast(&self) -> Option<&T>; }` does not really allow
//      for a lifetime in `T` that depends on the lifetime of the `&self` parameter.

/// The trait that plugins should implement in order to handle the given type of events.
pub trait EventHandler<E> {
    fn handle_event(&mut self, event: E);
}

#[derive(Clone, Copy)]
pub struct SysExEvent<'a> {
    data: &'a[u8]
}

impl<'a> SysExEvent<'a> {
    pub fn new(data: &'a[u8]) -> Self {
        Self{data}
    }
}

impl<'a> IsNot<RawMidiEvent> for SysExEvent<'a>{}

impl_downcast!('a, SysExEvent<'a>);

/// A raw midi event.
/// Use this when you need to be able to clone the event.
#[derive(Clone, Copy)]
pub struct RawMidiEvent {
    data: [u8; 3]
}

impl RawMidiEvent {
    pub fn new(data: [u8; 3]) -> Self {
        Self {data}
    }
    pub fn data(&self) -> &[u8; 3] {
        &self.data
    }
}

impl_downcast!(RawMidiEvent);
/*
impl DowncastCheck<RawMidiEvent> for RawMidiEvent {
    fn can_downcast(&self) -> bool {
        true
    }
}

impl<T> DowncastCheck<T> for RawMidiEvent where T: IsNot<RawMidiEvent> {
    fn can_downcast(&self) -> bool {
        false
    }
}

impl Downcast<RawMidiEvent> for RawMidiEvent {
    fn downcast(self) -> Option<RawMidiEvent> {
        Some(self)
    }
}

impl<T> Downcast<T> for RawMidiEvent where T: IsNot<RawMidiEvent> {
    fn downcast(self) -> Option<T> {
        None
    }
}

impl DowncastRef<RawMidiEvent> for RawMidiEvent {
    fn downcast_ref(&self) -> Option<&RawMidiEvent> {
        Some(self)
    }
}

impl<T> DowncastRef<T> for RawMidiEvent where T: IsNot<RawMidiEvent> {
    fn downcast_ref(&self) -> Option<&T> {
        None
    }
}

impl DowncastMut<RawMidiEvent> for RawMidiEvent {
    fn downcast_mut(&mut self) -> Option<&mut RawMidiEvent> {
        Some(self)
    }
}

impl<T> DowncastMut<T> for RawMidiEvent where T: IsNot<RawMidiEvent> {
    fn downcast_mut(&mut self) -> Option<&mut T> {
        None
    }
}

*/

impl<'a> IsNot<SysExEvent<'a>> for RawMidiEvent {}

pub struct Timed<E> {
    pub time_in_frames: u32,
    pub event: E
}

impl<E> IsNot<RawMidiEvent> for Timed<E> {}
impl<'a, E> IsNot<SysExEvent<'a>> for Timed<E> {}
impl<E, EE> IsNot<Timed<EE>> for Timed<E> where E: IsNot<EE> {}
impl<E> Clone for Timed<E> where E: Clone {
    fn clone(&self) -> Self {
        Timed{
            time_in_frames: self.time_in_frames,
            event: self.event.clone()
        }
    }
}
impl<E> Copy for Timed<E> where E: Copy {}

//impl_downcast!(E, Timed<E>);
impl<E> DowncastCheck<Timed<E>> for Timed<E> {
    fn can_downcast(&self) -> bool {
        true;
    }
}

impl<E, T> DowncastCheck<T> for Timed<E> where T: IsNot<Timed<E>> {
    fn can_downcast(&self) -> bool {
        false;
    }
}