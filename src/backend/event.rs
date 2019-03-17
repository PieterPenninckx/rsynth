use std::any::Any;

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

pub trait Event: AsAny {
    /// The name of the type.
    /// This method probably allocates memory, so it should only be used for logging purposes.
    fn type_name(&self) -> String {
        "(unnamed Event)".to_string()
    }
}

impl<'a> dyn Event + 'a {
    pub fn downcast_ref<E: Event + 'static>(&self) -> Option<&E> {
        self.as_any().downcast_ref::<E>()
    }
}

pub struct RawMidiEvent {
    pub data: Vec<u8>
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

pub struct Timed<E> {
    pub time_in_samples: u32,
    pub event: E
}

impl<E: AsAny + 'static> AsAny for Timed<E> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
impl<E: Event + 'static> Event for Timed<E> {
    fn type_name(&self) -> String {
        format!("{}<{}>",stringify!(Timed), self.event.type_name())
    }
}

/// Syntactic sugar for dealing with &dyn Event
///
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
#[macro_export]
macro_rules! match_event{
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