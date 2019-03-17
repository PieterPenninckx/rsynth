use std::any::Any;

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

/// Trait used to represent events.
pub trait Event: AsAny {
    /// The name of the type.
    /// This method probably allocates memory, so it should only be used for logging purposes.
    fn type_name(&self) -> String {
        "(unnamed Event)".to_string()
    }
}

impl<'a> dyn Event + 'a {
    /// Convenience function to try to downcast to a given specific type.
    pub fn downcast_ref<E: Event + 'static>(&self) -> Option<&E> {
        self.as_any().downcast_ref::<E>()
    }
}

pub struct RawMidiEvent {
    // TODO: make sure that this can implement Copy.
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

pub trait EventHandler<E: Event> {
    fn handle_event(&mut self, event: &E);
}


#[macro_export]
macro_rules! dispatch_event{
    ($self_:expr; $event:expr; $t:ty) => {
        if let Some(e) = $event.downcast_ref::<$t>() {
            EventHandler::<$t>::handle_event($self_, e);
        }
    };
    ($self_:expr; $event:expr; $thead:ty $(, $ttail:ty)*) => {
        if let Some(e) = $event.downcast_ref::<$thead>() {
            EventHandler::<$thead>::handle_event($self_, e);
        }
        $(
        else if let Some(e) = $event.downcast_ref::<$ttail>() {
            EventHandler::<$ttail>::handle_event($self_, e);
        }
        )*
    }
}

#[cfg(test)]
mod test_event_dispatch {
    use super::{AsAny, Event, EventHandler};
    use std::any::Any;
    use asprim::AsPrim;
    use num_traits::Float;
    use super::super::{Plugin};

    #[allow(dead_code)]
    struct E1 {}
    impl E1 {
        #[allow(dead_code)]
        fn f1(&self) {
        }
    }
    
    impl AsAny for E1 {
       fn as_any(&self) -> &dyn Any { self }
    }
    
    impl Event for E1 {}
    
    #[allow(dead_code)]
    struct E2 {}
    
    impl E2 {
        #[allow(dead_code)]
        fn f2(&self) {
        }
    }
    impl AsAny for E2 {
       fn as_any(&self) -> &dyn Any { self }
    }
    
    impl Event for E2 {}
    
    #[allow(dead_code)]
    struct MyPlugin {}
    
    impl EventHandler<E1> for MyPlugin {
        fn handle_event(&mut self, event: &E1) {
            unimplemented!();
        }
    }
    
    impl EventHandler<E2> for MyPlugin {
        fn handle_event(&mut self, event: &E2) {
            unimplemented!();
        }
    }
    
    impl Plugin for MyPlugin {
        const NAME: &'static str = "";
        const MAX_NUMBER_OF_AUDIO_INPUTS: usize = 0;
        const MAX_NUMBER_OF_AUDIO_OUTPUTS: usize = 0;
        fn audio_input_name(index: usize) -> String {
            unimplemented!();
        }
        fn audio_output_name(index: usize) -> String {
            unimplemented!();
        }
        fn set_sample_rate(&mut self, sample_rate: f64) {
            unimplemented!();
        }
        fn render_buffer<F>(&mut self, inputs: &[&[F]], outputs: &mut [&mut [F]])
        where
            F: Float + AsPrim
        {
            unimplemented!();
        }
        fn handle_event(&mut self, event: &dyn Event) {
            dispatch_event!(self; event; E1);
            dispatch_event!(self; event; E1, E2);
        }
    }
}
