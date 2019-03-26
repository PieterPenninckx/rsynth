use dev_utilities::is_not::{IsNot, NotInRSynth};
use downcast::{Downcast, DowncastRef, DowncastMut, DowncastCheck};
use dev_utilities::specialize::Specialize;
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

impl<'a> IsNot<SysExEvent<'a>> for RawMidiEvent {}

pub struct Timed<E> {
    pub time_in_frames: u32,
    pub event: E
}

pub trait WithTime {
    fn time_in_frames(&self) -> Option<u32> {
        None
    }
}

impl<E> IsNot<RawMidiEvent> for Timed<E> {}
impl WithTime for RawMidiEvent {}

impl<'a, E> IsNot<SysExEvent<'a>> for Timed<E> {}
impl<'a> WithTime for SysExEvent<'a> {}

impl<E, EE> IsNot<Timed<EE>> for Timed<E> where E: IsNot<EE> {}
impl<E> WithTime for Timed<E> {
    fn time_in_frames(&self) -> Option<u32> {
        Some(self.time_in_frames)
    }
}
impl<E, EE> IsNot<Timed<E>> for EE where EE: NotInRSynth {}

impl<E> Clone for Timed<E> where E: Clone {
    fn clone(&self) -> Self {
        Timed{
            time_in_frames: self.time_in_frames,
            event: self.event.clone()
        }
    }
}

impl<E> Copy for Timed<E> where E: Copy {}

impl<E, EE> DowncastCheck<EE> for Timed<E> 
where 
    E: DowncastCheck<EE> 
{
    fn can_downcast(&self) -> bool {
        self.event.can_downcast()
    }
}

impl<E, EE> Downcast<EE> for Timed<E>
where
    E: Downcast<EE> 
{
    fn downcast(self) -> Option<EE> {
        self.event.downcast()
    }
}

impl<E, EE> DowncastRef<EE> for Timed<E>
where 
    E: DowncastRef<EE> 
{
    fn downcast_ref(&self) -> Option<&EE> {
        self.event.downcast_ref()
    }
}

impl<E, EE> DowncastMut<EE> for Timed<E>
where
    E: DowncastMut<EE>
{
    fn downcast_mut(&mut self) -> Option<&mut EE> {
        self.event.downcast_mut()
    }
}

impl<E> Specialize<RawMidiEvent> for Timed<E> {}
impl<'a, E> Specialize<SysExEvent<'a>> for Timed<E> {}
impl<E, T> Specialize<T> for Timed<E> where T: NotInRSynth {}
impl<E, T> Specialize<Timed<T>> for Timed<E>
where E:Specialize<T>
{
    fn can_specialize(&self) -> bool {
        self.event.can_specialize()
    }

    fn specialize(self) -> Option<Timed<T>> {
        if self.event.can_specialize() {
            let Timed{time_in_frames, event} = self;
            Some(
                Timed {
                    time_in_frames,
                    event: event.specialize().unwrap() // TODO: more graceful error handling.
                }
            )
        } else {
            return None
        }
    }
}
