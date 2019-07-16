use crate::event::{EventHandler, RawMidiEvent, Timed};
use asprim::AsPrim;
use num_traits::Float;

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

pub struct ToneIdentifier {
    tone: u8,
}

use crate::event::raw_midi_event_event_types::*;

impl PolyphonicEvent<ToneIdentifier> for RawMidiEvent {
    fn event_type(&self) -> PolyphonicEventType<ToneIdentifier> {
        match self.data()[0] & 0xF0 {
            RAW_MIDI_EVENT_NOTE_OFF => PolyphonicEventType::ReleaseVoice(ToneIdentifier {
                tone: self.data()[1],
            }),
            RAW_MIDI_EVENT_NOTE_ON => PolyphonicEventType::AssignNewVoice(ToneIdentifier {
                tone: self.data()[1],
            }),
            RAW_MIDI_EVENT_NOTE_AFTERTOUCH => PolyphonicEventType::VoiceSpecific(ToneIdentifier {
                tone: self.data()[1],
            }),
            _ => PolyphonicEventType::Broadcast,
        }
    }
}

impl<Event, Identifier> PolyphonicEvent<Identifier> for Timed<Event>
where
    Event: PolyphonicEvent<Identifier>,
{
    fn event_type(&self) -> PolyphonicEventType<Identifier> {
        self.event.event_type()
    }
}

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
}

mod voice_stealer {
    use super::{PolyphonicEvent, PolyphonicEventType, Voice, VoiceAssignment};
    use crate::middleware::polyphony::VoiceStealer;
    use std::marker::PhantomData;

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum BasicState<VoiceIdentifier>
    where
        VoiceIdentifier: Copy + Eq,
    {
        Idle,
        Active(VoiceIdentifier),
    }

    struct AssignFirstIdleVoice<VoiceIdentifier>
    where
        VoiceIdentifier: Copy + Eq,
    {
        _phantom_voice_identifier: PhantomData<VoiceIdentifier>,
    }

    impl<VoiceIdentifier> AssignFirstIdleVoice<VoiceIdentifier>
    where
        VoiceIdentifier: Copy + Eq,
    {
        fn new() -> Self {
            Self {
                _phantom_voice_identifier: PhantomData,
            }
        }
    }

    impl<VoiceIdentifier> VoiceStealer for AssignFirstIdleVoice<VoiceIdentifier>
    where
        VoiceIdentifier: Copy + Eq,
    {
        type State = BasicState<VoiceIdentifier>;
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
                .position(|voice| voice.state() == BasicState::Active(identifier))
        }

        fn find_idle_voice<V>(&mut self, identifier: VoiceIdentifier, voices: &mut [V]) -> usize
        where
            V: Voice<Self::State>,
        {
            voices
                .iter()
                .position(|voice| voice.state() == BasicState::Idle)
                .unwrap_or(0)
        }
    }
}
