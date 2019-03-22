use std::any::Any;
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



enum DynamicEventInner<'a> {
    AnyEvent(&'a dyn Any),
    SysExEvent(&'a [u8]),
    TimedSysexEvent{time_in_frames: u32, sysex_data: &'a [u8]}
}

pub struct DynamicEvent<'a> {
    inner: DynamicEventInner<'a>
}

pub trait AsDynamicEvent {
    fn as_dynamic<'a>(&'a self) -> DynamicEvent<'a>;
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

impl<T> AsDynamicEvent for T where T: AsAny {
    fn as_dynamic<'a>(&'a self) -> DynamicEvent<'a> {
        DynamicEvent{
            inner: DynamicEventInner::AnyEvent(self.as_any())
        }
    }
}

pub struct SysExEvent<'a> {
    data: &'a[u8]
}

impl<'a> SysExEvent<'a> {
    pub fn new(data: &'a[u8]) -> Self {
        Self{data}
    }
}

impl<'b> AsDynamicEvent for SysExEvent<'b> {
    fn as_dynamic<'a>(&'a self) -> DynamicEvent<'a> {
        DynamicEvent{inner: DynamicEventInner::SysExEvent(self.data)}
    }
}

/// Trait used to represent events.
pub trait Event: AsDynamicEvent {
    /// The name of the type.
    /// This method probably allocates memory, so it should only be used for logging purposes.
    fn type_name(&self) -> String {
        "(unnamed Event)".to_string()
    }
}

pub trait DownCastableTo<T> {
    fn downcast(&self) -> Option<&T>;
}

impl<'a, E: Event + AsAny + 'static> DownCastableTo<E> for dyn Event + 'a {
    fn downcast(&self) -> Option<&E> {
        match self.as_dynamic().inner {
            DynamicEventInner::AnyEvent(e) => {
                e.downcast_ref::<E>()
            },
            _ => {
                return None;
            }
        }
    }
}

impl<'a> DownCastableTo<SysExEvent<'a>> for dyn Event + 'a {
    fn downcast(&'a self) -> Option<&SysExEvent<'a>> {
        match self.as_dynamic().inner {
            DynamicEventInner::SysExEvent(data) => {
                Some(&SysExEvent {data})
            },
            _ => {
                return None;
            }
        }
    }
}

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

impl AsAny for RawMidiEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Event for RawMidiEvent {
    fn type_name(&self) -> String {
        stringify!(RawMidiEvent).to_string()
    }
}

/*
/// A system exclusive event.
/// Use `RawMidiEvent` when you want to be able to clone the event.
pub struct SysexEvent {
    data: Vec<u8>
}

impl SysexEvent {
    pub fn new(data: Vec<u8>) -> Self {
        Self {data}
    }
    pub fn data(&self) -> &[u8] {
        &self.data
    }
    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }
}

impl AsAny for SysexEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Event for SysexEvent {
    fn type_name(&self) -> String {
        stringify!(SysexEvent).to_string()
    }
}
*/

pub struct Timed<E> {
    pub time_in_frames: u32,
    pub event: E
}

impl<E: AsAny + 'static> AsAny for Timed<E> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<E: Event + AsAny + 'static> Event for Timed<E> {
    fn type_name(&self) -> String {
        format!("{}<{}>",stringify!(Timed), self.event.type_name())
    }
}

impl<'a> AsDynamicEvent for Timed<SysExEvent<'a>> {
    fn as_dynamic<'b>(&'b self) -> DynamicEvent<'b> {
        DynamicEvent{
            inner: DynamicEventInner::TimedSysexEvent {time_in_frames: self.time_in_frames, sysex_data: self.event.data}
        }
    }
}

impl<'a> Event for Timed<SysExEvent<'a>> {
    fn type_name(&self) -> String {
        format!("{}<{}>",stringify!(Timed), self.event.type_name())
    }
}

impl<'a> DownCastableTo<Timed<SysExEvent<'a>>> for dyn Event + 'a {
    fn downcast(&self) -> Option<&Timed<SysExEvent<'a>>> {
        match self.as_dynamic().inner {
            DynamicEventInner::TimedSysexEvent {time_in_frames, sysex_data} => {
                Some(Timed{time_in_frames, event: SysExEvent{data: sysex_data}})
            },
            _ => {
                return None;
            }
        }
    }
}


/// Syntactic sugar for dealing with &dyn Event
///
/// Example:
/// ```
/// # #[macro_use] extern crate rsynth;
/// # #[macro_use] extern crate log;
/// # use rsynth::backend::event::{AsAny, Event};
/// # use std::any::Any;
/// struct EventTypeOne {}
/// impl EventTypeOne {
///     fn say_one(&self) {
///         println!("One");
///     }
/// }
/// # impl AsAny for EventTypeOne {
/// #    fn as_any(&self) -> &dyn Any { self }
/// # }
/// impl Event for EventTypeOne {}
///
/// struct EventTypeTwo {}
/// impl EventTypeTwo {
///     fn say_two(&self) {
///         println!("Two");
///     }
/// }
/// # impl AsAny for EventTypeTwo {
/// #    fn as_any(&self) -> &dyn Any { self }
/// # }
/// impl Event for EventTypeTwo {}
///
/// fn handle_event(event: &dyn Event) {
///     match_event!(
///         (event) {
///             (event_of_type_1: EventTypeOne) => {
///                 event_of_type_1.say_one()
///             },
///             (event_of_type_2: EventTypeTwo) => {
///                 event_of_type_2.say_two()
///             },
///             _ => {
///                 warn!("Ignoring event of unhandled type {}", event.type_name());
///             }
///         }
///     )
/// }
/// ```
/// The `_ => {}` arm is optional, but should occur last.

#[macro_export]
macro_rules! match_event {
    (
        ($event:expr)
        {
            ($sub:ident : $subtype:ty) => $b:block,
            _ => $elseblock:block
        }
    ) => {
        if let Some($sub) = $event.downcast_ref::<$subtype>()
            $b
        else
            $elseblock
    };
    (
        ($event:expr)
        {
            ($sub:ident : $subtype:ty) => $b:block
        }
    ) => {
        if let Some($sub) = $event.downcast_ref::<$subtype>() $b
    };
    (($event:expr)
        {
            ($headsub:ident : $headsubtype:ty) => $headblock:block
            $(, ($tailsub:ident : $tailsubtype:ty) => $tailblock:block )*
            , _ => $elseblock:block
        }
    ) => {
        if let Some($headsub) = $event.downcast_ref::<$headsubtype>() $headblock
        $(else if let Some($tailsub) = $event.as_any().downcast_ref::<$tailsubtype>() $tailblock)*
        else $elseblock
    };
    (($event:expr)
        {
            ($headsub:ident : $headsubtype:ty) => $headblock:block
            $(, ($tailsub:ident : $tailsubtype:ty) => $tailblock:block )*
        }
    ) => {
        if let Some($headsub) = $event.downcast_ref::<$headsubtype>() $headblock
        $(else if let Some($tailsub) = $event.as_any().downcast_ref::<$tailsubtype>() $tailblock)*
    };
}

#[cfg(test)]
mod test_event {
    use super::{AsAny, Event};
    use std::any::Any;
    
    struct E1 {}
    impl E1 {
        fn f1(&self) {
        }
    }
    
    impl AsAny for E1 {
       fn as_any(&self) -> &dyn Any { self }
    }
    
    impl Event for E1 {}
    
    struct E2 {}
    
    impl E2 {
        fn f2(&self) {
        }
    }
    impl AsAny for E2 {
       fn as_any(&self) -> &dyn Any { self }
    }
    
    impl Event for E2 {}
    
    #[allow(dead_code)]
    fn handle_event_one_block_without_wildcard(event: &dyn Event) {
        match_event!(
            (event) {
                (e1: E1) => {
                    e1.f1()
                }
            }
        )
    }
    
    #[allow(dead_code)]
    fn handle_event_one_block_with_wildcard(event: &dyn Event) {
        match_event!(
            (event) {
                (e1: E1) => {
                    e1.f1()
                },
                _ => {
                }
            }
        )
    }
    
    #[allow(dead_code)]
    fn handle_event_two_blocks_without_wildcard(event: &dyn Event) {
        match_event!(
            (event) {
                (e1: E1) => {
                    e1.f1()
                },
                (e2: E2) => {
                    e2.f2()
                },
                _ => {
                }
            }
        )
    }
    
    // two blocks with wildcard is in the doc-test
}
