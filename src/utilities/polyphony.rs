//! Polyphony consists of different steps:
//!
//! 1. Classify how the event should be dispatched.
//!    How exactly it should be classified, is defined by the `EventDispatchClass` enum.
//!    The dispatching itself is done by a type that implements the `EventDispatchClassifier` trait.
//! 2. Next, a voice should be assigned to the event.
//!    The `VoiceAssigner` trait defines this action.
//! 3. Then, the event can be dispatched.
//!    The `EventDispatcher` trait and the `ContextualEventDispatcher` trait define
//!    methods for doing this.
use crate::event::{ContextualEventHandler, EventHandler, RawMidiEvent};

pub enum EventDispatchClass<Identifier> {
    Broadcast,
    AssignNewVoice(Identifier),
    VoiceSpecific(Identifier),
    ReleaseVoice(Identifier),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ToneIdentifier(pub u8);

pub trait EventDispatchClassifier<Event>
where
    Event: Copy,
{
    type VoiceIdentifier: Eq + Copy;

    fn classify(&self, event: &Event) -> EventDispatchClass<Self::VoiceIdentifier>;
}

pub struct RawMidiEventToneIdentifierDispatchClassifier;

impl<Event> EventDispatchClassifier<Event> for RawMidiEventToneIdentifierDispatchClassifier
where
    Event: AsRef<RawMidiEvent> + Copy,
{
    type VoiceIdentifier = ToneIdentifier;

    fn classify(&self, event: &Event) -> EventDispatchClass<Self::VoiceIdentifier> {
        let data = event.as_ref().data();
        use crate::event::raw_midi_event_event_types::*;
        match data[0] & 0xF0 {
            RAW_MIDI_EVENT_NOTE_OFF => EventDispatchClass::ReleaseVoice(ToneIdentifier(data[1])),
            RAW_MIDI_EVENT_NOTE_ON => {
                if data[2] == 0 {
                    // Velocity 0 is considered the same as note off.
                    EventDispatchClass::ReleaseVoice(ToneIdentifier(data[1]))
                } else {
                    EventDispatchClass::AssignNewVoice(ToneIdentifier(data[1]))
                }
            }
            RAW_MIDI_EVENT_NOTE_AFTERTOUCH => {
                EventDispatchClass::VoiceSpecific(ToneIdentifier(data[1]))
            }
            _ => EventDispatchClass::Broadcast,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VoiceAssignment {
    None,
    All,
    Some(usize),
}

pub trait Voice<State> {
    fn state(&self) -> State;
}

pub trait VoiceAssigner<Event>: EventDispatchClassifier<Event>
where
    Event: Copy,
{
    type Voice;

    fn assign_event(&mut self, event: Event, voices: &mut [Self::Voice]) -> VoiceAssignment {
        match self.classify(&event) {
            EventDispatchClass::Broadcast => VoiceAssignment::All,
            EventDispatchClass::VoiceSpecific(identifier)
            | EventDispatchClass::ReleaseVoice(identifier) => {
                match self.find_active_voice(identifier, voices) {
                    Some(index) => VoiceAssignment::Some(index),
                    None => VoiceAssignment::None,
                }
            }
            EventDispatchClass::AssignNewVoice(identifier) => {
                VoiceAssignment::Some(self.find_idle_voice(identifier, voices))
            }
        }
    }

    fn find_active_voice(
        &mut self,
        identifier: Self::VoiceIdentifier,
        voices: &mut [Self::Voice],
    ) -> Option<usize>;

    fn find_idle_voice(
        &mut self,
        identifier: Self::VoiceIdentifier,
        voices: &mut [Self::Voice],
    ) -> usize;
}

pub trait EventDispatcher<Event>: VoiceAssigner<Event>
where
    Event: Copy,
    Self::Voice: EventHandler<Event>,
{
    fn dispatch_event(&mut self, event: Event, voices: &mut [Self::Voice]) {
        match self.assign_event(event, voices) {
            VoiceAssignment::None => {}
            VoiceAssignment::Some(index) => {
                voices[index].handle_event(event);
            }
            VoiceAssignment::All => {
                for voice in voices {
                    voice.handle_event(event);
                }
            }
        }
    }
}

pub trait ContextualEventDispatcher<Event, Context>: VoiceAssigner<Event>
where
    Event: Copy,
    Self::Voice: ContextualEventHandler<Event, Context>,
{
    fn dispatch_contextual_event(
        &mut self,
        event: Event,
        voices: &mut [Self::Voice],
        context: &mut Context,
    ) {
        match self.assign_event(event, voices) {
            VoiceAssignment::None => {}
            VoiceAssignment::Some(index) => {
                voices[index].handle_event(event, context);
            }
            VoiceAssignment::All => {
                for voice in voices {
                    voice.handle_event(event, context);
                }
            }
        }
    }
}

pub mod simple_event_dispatching {
    use super::{
        ContextualEventDispatcher, EventDispatchClass, EventDispatchClassifier, EventDispatcher,
        Voice, VoiceAssigner,
    };
    use crate::event::{ContextualEventHandler, EventHandler};
    use std::marker::PhantomData;

    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum SimpleVoiceState<VoiceIdentifier>
    where
        VoiceIdentifier: Copy + Eq,
    {
        Idle,
        Releasing(VoiceIdentifier),
        Active(VoiceIdentifier),
    }

    pub struct SimpleEventDispatcher<Classifier, V> {
        classifier: Classifier,
        _voice_phantom: PhantomData<V>,
    }

    impl<Classifier, V> SimpleEventDispatcher<Classifier, V> {
        pub fn new(classifier: Classifier) -> Self {
            Self {
                classifier,
                _voice_phantom: PhantomData,
            }
        }
    }

    impl<Event, Classifier, Voice> EventDispatchClassifier<Event>
        for SimpleEventDispatcher<Classifier, Voice>
    where
        Classifier: EventDispatchClassifier<Event>,
        Event: Copy,
    {
        type VoiceIdentifier = Classifier::VoiceIdentifier;

        fn classify(&self, event: &Event) -> EventDispatchClass<Self::VoiceIdentifier> {
            self.classifier.classify(event)
        }
    }

    impl<Event, Classifier, V> VoiceAssigner<Event> for SimpleEventDispatcher<Classifier, V>
    where
        Classifier: EventDispatchClassifier<Event>,
        V: Voice<SimpleVoiceState<Classifier::VoiceIdentifier>>,
        Event: Copy,
    {
        type Voice = V;

        fn find_active_voice(
            &mut self,
            identifier: Self::VoiceIdentifier,
            voices: &mut [Self::Voice],
        ) -> Option<usize> {
            voices
                .iter()
                .position(|voice| voice.state() == SimpleVoiceState::Active(identifier))
            // TODO: what should we do when we receive an event for a voice that
            // is already releasing?
        }

        fn find_idle_voice(
            &mut self,
            _identifier: Self::VoiceIdentifier,
            voices: &mut [Self::Voice],
        ) -> usize {
            let mut second_best = 0;
            for (index, voice) in voices.iter().enumerate() {
                match voice.state() {
                    SimpleVoiceState::Idle => {
                        return index;
                    }
                    SimpleVoiceState::Releasing(_) => {
                        second_best = index;
                    }
                    SimpleVoiceState::Active(_) => {}
                }
            }
            return second_best;
        }
    }

    impl<Event, Classifier, V, Context> ContextualEventDispatcher<Event, Context>
        for SimpleEventDispatcher<Classifier, V>
    where
        Classifier: EventDispatchClassifier<Event>,
        V: Voice<SimpleVoiceState<Classifier::VoiceIdentifier>>
            + ContextualEventHandler<Event, Context>,
        Event: Copy,
    {
    }

    impl<Event, Classifier, V> EventDispatcher<Event> for SimpleEventDispatcher<Classifier, V>
    where
        Classifier: EventDispatchClassifier<Event>,
        V: Voice<SimpleVoiceState<Classifier::VoiceIdentifier>> + EventHandler<Event>,
        Event: Copy,
    {
    }
}
