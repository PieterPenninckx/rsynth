//! Utility to facilitate genarating different sounds at the same time (polyphony).
//!
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
//!
//! # Example of using polyphony
//!
//! The following example illustrates a plugin (or application) that has multiple voices that
//! correspond to different tones.
//!
//! ```
//! use rsynth::utilities::polyphony::{Voice, EventDispatchClassifier, ToneIdentifier,
//!     RawMidiEventToneIdentifierDispatchClassifier, ContextualEventDispatcher};
//! use rsynth::utilities::polyphony::simple_event_dispatching::SimpleVoiceState;
//! use rsynth::utilities::polyphony::simple_event_dispatching::SimpleEventDispatcher;
//! use rsynth::event::{ContextualEventHandler, Indexed, Timed, RawMidiEvent};
//! use rsynth::ContextualAudioRenderer;
//! use rsynth::buffer::AudioBufferInOut;
//!
//! struct MyVoice {
//!     // ...
//! }
//!
//! impl Voice<SimpleVoiceState<ToneIdentifier>> for MyVoice {
//!     fn state(&self) -> SimpleVoiceState<ToneIdentifier> {
//!         // Let the event dispatcher know what state this voice is in.
//!         unimplemented!();
//!     }
//! }
//!
//! impl<Context> ContextualEventHandler<Timed<RawMidiEvent>, Context> for MyVoice {
//!     fn handle_event(&mut self, event: Timed<RawMidiEvent>, context: &mut Context) {
//!         // Here you typically change the state of the voice.
//!         unimplemented!()
//!     }
//! }
//!
//! impl<Context> ContextualAudioRenderer<f32, Context> for MyVoice {
//!     fn render_buffer(&mut self, buffer: &mut AudioBufferInOut<f32>, context: &mut Context) {
//!         // Render one voice.
//!         unimplemented!()
//!     }
//! }
//!
//! struct MyPlugin {
//!     voices: Vec<MyVoice>,
//!     // ...
//! }
//!
//! impl<Context> ContextualEventHandler<Indexed<Timed<RawMidiEvent>>, Context> for MyPlugin
//! {
//!     fn handle_event(&mut self, event: Indexed<Timed<RawMidiEvent>>, context: &mut Context) {
//!         let mut dispatcher = SimpleEventDispatcher::new(RawMidiEventToneIdentifierDispatchClassifier);
//!         // Here we simply pass the context that we're given, but you can also pass a custom
//!         // context that uses shared data that is stored in `self`.
//!         dispatcher.dispatch_contextual_event(event.event, &mut self.voices, context);
//!     }
//! }
//!
//! impl<Context> ContextualAudioRenderer<f32, Context> for MyPlugin
//! {
//!     fn render_buffer(&mut self, buffer: &mut AudioBufferInOut<f32>, context: &mut Context) {
//!         for voice in self.voices.iter_mut() {
//!             // Here we simply pass the context that we're given, but you can also pass a custom
//!             // context that uses shared data that is stored in `self`.
//!             voice.render_buffer(buffer, context);
//!         }
//!     }
//! }
//!
//! ```

use crate::event::{ContextualEventHandler, EventHandler, RawMidiEvent};
use midi_consts::channel_event::*;

pub enum EventDispatchClass<Identifier> {
    Broadcast,
    AssignNewVoice(Identifier),
    VoiceSpecific(Identifier),
    ReleaseVoice(Identifier),
}

/// Used to dispatch polyphonic event to the correct voice, based on the tone of the event.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ToneIdentifier(pub u8);

pub trait EventDispatchClassifier<Event>
where
    Event: Copy,
{
    type VoiceIdentifier: Eq + Copy;

    fn classify(&self, event: &Event) -> EventDispatchClass<Self::VoiceIdentifier>;
}

#[derive(Default)]
pub struct RawMidiEventToneIdentifierDispatchClassifier;

impl<Event> EventDispatchClassifier<Event> for RawMidiEventToneIdentifierDispatchClassifier
where
    Event: AsRef<RawMidiEvent> + Copy,
{
    type VoiceIdentifier = ToneIdentifier;

    fn classify(&self, event: &Event) -> EventDispatchClass<Self::VoiceIdentifier> {
        let data = event.as_ref().data();
        match data[0] & EVENT_TYPE_MASK {
            NOTE_OFF => EventDispatchClass::ReleaseVoice(ToneIdentifier(data[1])),
            NOTE_ON => {
                if data[2] == 0 {
                    // Velocity 0 is considered the same as note off.
                    EventDispatchClass::ReleaseVoice(ToneIdentifier(data[1]))
                } else {
                    EventDispatchClass::AssignNewVoice(ToneIdentifier(data[1]))
                }
            }
            POLYPHONIC_KEY_PRESSURE => EventDispatchClass::VoiceSpecific(ToneIdentifier(data[1])),
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

/// Implement this trait to inform the polyphonic event dispatcher what state this voice is in.
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
    /// Dispatch an event to the voice or voices that should handle it.
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

/// Some basic event dispatching.
pub mod simple_event_dispatching {
    use super::{
        ContextualEventDispatcher, EventDispatchClass, EventDispatchClassifier, EventDispatcher,
        Voice, VoiceAssigner,
    };
    use crate::event::{ContextualEventHandler, EventHandler};
    use std::marker::PhantomData;

    /// A simple voice state
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum SimpleVoiceState<VoiceIdentifier>
    where
        VoiceIdentifier: Copy + Eq,
    {
        /// The voice is idle (in other words: doing nothing).
        Idle,
        /// The voice has received a signal to stop, but is still rendering audio (e.g. some reverb
        /// after the end of the audio).
        ///
        /// The `VoiceIdentifier` indicates what it is still rendering.
        Releasing(VoiceIdentifier),
        /// The voice has not yet received a signal to stop and is still rendering audio.
        Active(VoiceIdentifier),
    }

    /// A simple event dispatcher.
    ///
    /// The type parameter `Classifier` refers to the classifier that is used to classify events.
    /// In order to use this `SimpleEventDispatcher`,
    /// the concrete type used for `Classifier` should implement the `EventDispatchClassifier` trait.
    ///
    /// The type parameter `V` refers to the voice.
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

    impl<Classifier, V> Default for SimpleEventDispatcher<Classifier, V>
    where
        Classifier: Default,
    {
        fn default() -> Self {
            Self {
                classifier: Classifier::default(),
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
            second_best
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
