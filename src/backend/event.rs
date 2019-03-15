use std::any::Any;

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

pub trait Event: AsAny {
}

pub struct RawMidiEvent {
    pub data: Vec<u8>,
}

impl AsAny for RawMidiEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
impl Event for RawMidiEvent {}

pub struct Timed<E> {
    pub time_in_samples: u32,
    pub event: E
}

impl<E: AsAny + 'static> AsAny for Timed<E> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
impl<E: Event + 'static> Event for Timed<E> {}

/// Syntactic sugar for dealing with &dyn Event
///
/// ```
/// # #[macro_use] extern crate rsynth;
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
///             }
///         }
///     )
/// }
/// ```
#[macro_export]
macro_rules! match_event{
    (($event:expr) {($sub:ident : $subtype:ty) => $b:block }) => {
        if let Some($sub) = $event.as_any().downcast_ref::<$subtype>() $b
    };
    (($event:expr) {($headsub:ident : $headsubtype:ty) => $headblock:block
        $(, ($tailsub:ident : $tailsubtype:ty) => $tailblock:block )*  }) => {
        if let Some($headsub) = $event.as_any().downcast_ref::<$headsubtype>() $headblock
        $(else if let Some($tailsub) = $event.as_any().downcast_ref::<$tailsubtype>() $tailblock)*
    }
}