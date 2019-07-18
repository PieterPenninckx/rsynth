use crate::event::{ContextualEventHandler, EventHandler, RawMidiEvent, Timed};

pub trait Voice<State> {
    fn state(&self) -> State;
}

pub enum PolyphonicEventType<Identifier> {
    Broadcast,
    AssignNewVoice(Identifier),
    VoiceSpecific(Identifier),
    ReleaseVoice(Identifier),
}

pub trait PolyphonicEvent<Identifier>: Copy {
    fn event_type(&self) -> PolyphonicEventType<Identifier>;
}

impl<Event, Identifier> PolyphonicEvent<Identifier> for Timed<Event>
where
    Event: PolyphonicEvent<Identifier>,
{
    fn event_type(&self) -> PolyphonicEventType<Identifier> {
        self.event.event_type()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ToneIdentifier(pub u8);

use crate::event::raw_midi_event_event_types::*;

impl PolyphonicEvent<ToneIdentifier> for RawMidiEvent {
    fn event_type(&self) -> PolyphonicEventType<ToneIdentifier> {
        match self.data()[0] & 0xF0 {
            RAW_MIDI_EVENT_NOTE_OFF => {
                PolyphonicEventType::ReleaseVoice(ToneIdentifier(self.data()[1]))
            }
            RAW_MIDI_EVENT_NOTE_ON => {
                if self.data()[2] == 0 {
                    // Velocity 0 is considered the same as note off.
                    PolyphonicEventType::ReleaseVoice(ToneIdentifier(self.data()[1]))
                } else {
                    PolyphonicEventType::AssignNewVoice(ToneIdentifier(self.data()[1]))
                }
            }
            RAW_MIDI_EVENT_NOTE_AFTERTOUCH => {
                PolyphonicEventType::VoiceSpecific(ToneIdentifier(self.data()[1]))
            }
            _ => PolyphonicEventType::Broadcast,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VoiceAssignment {
    None,
    All,
    Some(usize),
}

pub trait VoiceStealer {
    type State;
    type VoiceIdentifier;

    fn assign_event<Event, V>(&mut self, event: Event, voices: &mut [V]) -> VoiceAssignment
    where
        V: Voice<Self::State>,
        Event: PolyphonicEvent<Self::VoiceIdentifier>,
    {
        match event.event_type() {
            PolyphonicEventType::Broadcast => VoiceAssignment::All,
            PolyphonicEventType::VoiceSpecific(identifier)
            | PolyphonicEventType::ReleaseVoice(identifier) => {
                match self.find_active_voice(identifier, voices) {
                    Some(index) => VoiceAssignment::Some(index),
                    None => VoiceAssignment::None,
                }
            }
            PolyphonicEventType::AssignNewVoice(identifier) => {
                VoiceAssignment::Some(self.find_idle_voice(identifier, voices))
            }
        }
    }

    fn find_active_voice<V>(
        &mut self,
        identifier: Self::VoiceIdentifier,
        voices: &mut [V],
    ) -> Option<usize>
    where
        V: Voice<Self::State>;

    fn find_idle_voice<V>(&mut self, identifier: Self::VoiceIdentifier, voices: &mut [V]) -> usize
    where
        V: Voice<Self::State>;

    fn dispatch_event<Event, V>(&mut self, event: Event, voices: &mut [V])
    where
        V: Voice<Self::State> + EventHandler<Event>,
        Event: PolyphonicEvent<Self::VoiceIdentifier>,
    {
        let assignment = self.assign_event(event, voices);
        match assignment {
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

    fn dispatch_contextual_event<Event, V, Context>(
        &mut self,
        event: Event,
        voices: &mut [V],
        context: &mut Context,
    ) where
        V: Voice<Self::State> + ContextualEventHandler<Event, Context>,
        Event: PolyphonicEvent<Self::VoiceIdentifier>,
    {
        let assignment = self.assign_event(event, voices);
        match assignment {
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

pub mod voice_stealer {
    use super::{Voice, VoiceStealer};
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

    pub struct SimpleVoiceStealer<VoiceIdentifier>
    where
        VoiceIdentifier: Copy + Eq,
    {
        _phantom_voice_identifier: PhantomData<VoiceIdentifier>,
    }

    impl<VoiceIdentifier> SimpleVoiceStealer<VoiceIdentifier>
    where
        VoiceIdentifier: Copy + Eq,
    {
        pub fn new() -> Self {
            Self {
                _phantom_voice_identifier: PhantomData,
            }
        }
    }

    impl<VoiceIdentifier> VoiceStealer for SimpleVoiceStealer<VoiceIdentifier>
    where
        VoiceIdentifier: Copy + Eq,
    {
        type State = SimpleVoiceState<VoiceIdentifier>;
        type VoiceIdentifier = VoiceIdentifier;

        fn find_active_voice<V>(
            &mut self,
            identifier: VoiceIdentifier,
            voices: &mut [V],
        ) -> Option<usize>
        where
            V: Voice<Self::State>,
        {
            voices
                .iter()
                .position(|voice| voice.state() == SimpleVoiceState::Active(identifier))
            // TODO: what should we do when we receive an event for a voice that
            // is already releasing?
        }

        fn find_idle_voice<V>(&mut self, _identifier: VoiceIdentifier, voices: &mut [V]) -> usize
        where
            V: Voice<Self::State>,
        {
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
}
