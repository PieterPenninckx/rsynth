use asprim::AsPrim;
use note::*;
use num_traits::Float;
use backend::Plugin;
use backend::{Event, RawMidiEvent, Transparent};
use std::marker::PhantomData;
use std::default::Default;

pub trait Voice {
    fn is_playing(&self) -> bool;
}

#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub struct VoiceWithState<V, S> {
    voice: V,
    state: S
}

/// Implement this trait to define your own `VoiceStealMode`.
// Ideally, a `VoiceStealMode` "contains" the voices instead of borrowing them in the
// `find_idle_voice` function and the `find_voice_playing_note` functions,
// but that would require higher kinded types...
pub trait VoiceStealMode
{
    /// The type of the voice (implementing the `Voice` trait) that this VoiceStealMode can handle.
    type V;

    /// A data type to store the state of the voice (idle, ...) if needed.
    type State: Sized + Default;

    /// Decide which voice should handle a given note.
    /// This method is at the heart of the `VoiceStealMode`.
    fn find_idle_voice<'v>(&mut self, voices: &'v mut[VoiceWithState<Self::V, Self::State>], note: u8) -> &'v mut VoiceWithState<Self::V, Self::State>;

    /// Return the voice that is playing a given note, if any.
    fn find_voice_playing_note<'v>(&mut self, voices: &'v mut [VoiceWithState<Self::V, Self::State>], note: u8) -> Option<&'v mut VoiceWithState<Self::V, Self::State>>;

    /// Mark this voice as "active".
    fn mark_voice_as_active(&mut self, &mut VoiceWithState<Self::V, Self::State>, note: u8);

    /// Mark the given voice as "inactive".
    fn mark_voice_as_inactive(&mut self, &mut VoiceWithState<Self::V, Self::State>);
}

/// `Polyphonic` is middleware that adds polyphony.
pub struct Polyphonic<Vc, VSM: VoiceStealMode<V=Vc>>
{
    voices: Vec<VoiceWithState<Vc, VSM::State>>,
    voice_steal_mode: VSM
}

impl<Vc,VSM> Polyphonic<Vc, VSM>
    where VSM: VoiceStealMode<V=Vc>
{
    pub fn new(voice_steal_mode: VSM, voices: Vec<Vc>) -> Self {
        let voices_with_states = voices
            .into_iter()
            .map(
                |v| {
                    VoiceWithState{
                        voice: v,
                        state: VSM::State::default()
                    }
                }
            )
            .collect();
        Polyphonic{
            voices: voices_with_states,
            voice_steal_mode
        }
    }
}

impl<'e, Vc, VSM, U> Plugin<Event<RawMidiEvent<'e>, U>> for Polyphonic<Vc, VSM>
where
    VSM: VoiceStealMode<V=Vc>,
    Vc: Voice,
    for<'a> VSM::V: Plugin<Event<RawMidiEvent<'a>, U>>
{
    const NAME: &'static str = Vc::NAME;
    const MAX_NUMBER_OF_AUDIO_INPUTS: usize = Vc::MAX_NUMBER_OF_AUDIO_INPUTS;
    const MAX_NUMBER_OF_AUDIO_OUTPUTS: usize = Vc::MAX_NUMBER_OF_AUDIO_OUTPUTS;

    fn audio_input_name(index: usize) -> String {
        Vc::audio_input_name(index)
    }

    fn audio_output_name(index: usize) -> String {
        Vc::audio_output_name(index)
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        for voice in self.voices.iter_mut() {
            voice.voice.set_sample_rate(sample_rate);
        }
    }

    fn render_buffer<F>(&mut self, inputs: &[&[F]], outputs: &mut[&mut[F]])
    where F: Float + AsPrim
    {
        for output in outputs.iter_mut() {
            for v in output.iter_mut() {
                *v = F::zero();
            }
        }
        for mut voice in self.voices.iter_mut() {
            if voice.voice.is_playing() {
                voice.voice.render_buffer::<F>(inputs, outputs);
            }
        }
    }
    
    fn handle_event(&mut self, event: &Event<RawMidiEvent<'e>, U>) {
        if let Event::Timed{samples: _, event: raw} = event {
            let note_data = NoteData::data(raw.data);
            let mut voice;
            if note_data.state == NoteState::On {
                let v = self.voice_steal_mode.find_idle_voice(&mut self.voices, note_data.note);
                self.voice_steal_mode.mark_voice_as_active(v, note_data.note);
                voice = &mut v.voice;
            } else {
                match self.voice_steal_mode.find_voice_playing_note(&mut self.voices, note_data.note) {
                    Some(v) => {
                        if note_data.state == NoteState::Off {
                            self.voice_steal_mode.mark_voice_as_inactive(v);
                        }
                        voice = &mut v.voice;
                    },
                    None => {
                        return;
                    }
                }
            }
            voice.handle_event(&event);
        } else {
            for mut voice in self.voices.iter_mut() {
                voice.voice.handle_event(&event);
            }
        }
    }
}

impl<Vc, VSM> Transparent for Polyphonic<Vc, VSM>
where VSM : VoiceStealMode<V=Vc>
{
    type Inner = Vc;

    fn get(&self) -> &<Self as Transparent>::Inner {
        &self.voices[0].voice
    }

    fn get_mut(&mut self) -> &mut <Self as Transparent>::Inner {
        &mut self.voices[0].voice
    }
}

#[derive(PartialEq, Eq)]
#[derive(Debug)]
enum PlayingState {
    Playing(u8),
    Off
}

impl Default for PlayingState {
    fn default() -> Self {
        PlayingState::Off
    }
}

#[derive(PartialEq, Eq)]
#[derive(Debug)]
pub struct SimpleVoiceStealerState {
    is_releasing: bool,
    playing_state: PlayingState
}

impl Default for SimpleVoiceStealerState {
    fn default() -> Self {
        SimpleVoiceStealerState {
            is_releasing: false,
            playing_state: PlayingState::default()
        }
    }
}

/// A simple voice stealer algorithm that just returns an idle voice if it can find one
/// and otherwise returns an arbitrary voice.
pub struct SimpleVoiceStealer<V> {
    _voices: PhantomData<V>
}

impl<V> SimpleVoiceStealer<V> {
    pub fn new() -> Self {
        SimpleVoiceStealer {
            _voices: PhantomData
        }
    }
}

impl<V: Voice> SimpleVoiceStealer<V> {
    fn mark_finished_if_needed(voice: &mut VoiceWithState<<Self as VoiceStealMode>::V, <Self as VoiceStealMode>::State>) {
        if ! voice.voice.is_playing() {
            voice.state.is_releasing = false;
            voice.state.playing_state = PlayingState::Off;
        }
    }
}

impl<Vc> VoiceStealMode for SimpleVoiceStealer<Vc>
    where Vc: Voice
{
    type V = Vc;
    type State = SimpleVoiceStealerState;

    fn find_idle_voice<'v>(&mut self, voices: &'v mut [VoiceWithState<Self::V, Self::State>], note: u8) -> &'v mut VoiceWithState<Self::V, Self::State> {
        let mut index = 0;
        for (i, voice) in voices.iter_mut().enumerate() {
            Self::mark_finished_if_needed(voice);
            if ! voice.voice.is_playing() {
                index = i;
                break;
            }
        }
        // TODO: The "stolen" voice should get a "stop playing" event before it is re-used.
        return &mut voices[index];
    }

    fn find_voice_playing_note<'v>(&mut self, voices: &'v mut [VoiceWithState<Self::V, Self::State>], note: u8) -> Option<&'v mut VoiceWithState<Self::V, Self::State>> {

        for voice in voices.iter_mut() {
            Self::mark_finished_if_needed(voice);
            if voice.state.playing_state == PlayingState::Playing(note) {
                return Some(voice);
            }
        }
        None
    }

    fn mark_voice_as_active(&mut self, voice: &mut VoiceWithState<Self::V, Self::State>, note: u8) {
        voice.state.is_releasing = false;
        voice.state.playing_state = PlayingState::Playing(note);
    }

    fn mark_voice_as_inactive(&mut self, voice: &mut VoiceWithState<<Self as VoiceStealMode>::V, Self::State>) {
        voice.state.is_releasing = true;
    }
}

#[cfg(test)]
mod simple_voice_stealer_tests {
    use super::VoiceWithState;
    use super::SimpleVoiceStealerState;
    use super::SimpleVoiceStealer;
    use super::VoiceStealMode;
    use super::Voice;
    use std::default::Default;

    #[derive(Default)]
    #[derive(Debug)]
    #[derive(PartialEq, Eq)]
    struct TestVoice {
        index: usize,
        is_playing: bool
    }

    impl Voice for TestVoice {
        fn is_playing(&self) -> bool {
            self.is_playing
        }
    }

    impl TestVoice {
        fn new(i: usize) -> Self {
            TestVoice {
                index: i,
                is_playing: false
            }
        }
    }

    #[test]
    fn test_simple_voice_stealer_find_idle_voice() {
        let number_of_voices = 3;
        let mut voices_with_state = vec![];
        let mut simple_voice_stealer = SimpleVoiceStealer::<TestVoice>::new();
        for i in 0..number_of_voices {
            voices_with_state.push(VoiceWithState {
                voice: TestVoice::new(i),
                state: SimpleVoiceStealerState::default()
            });
        }

        {
            let idle_voice = simple_voice_stealer.find_idle_voice(&mut voices_with_state, 0);
            assert_eq!(idle_voice.voice.index, 0);
        }
        voices_with_state[0].voice.is_playing = true;
        simple_voice_stealer.mark_voice_as_active(&mut voices_with_state[0], 0);
        {
            let idle_voice = simple_voice_stealer.find_idle_voice(&mut voices_with_state, 0);
            assert_eq!(idle_voice.voice.index, 1);
        }
        voices_with_state[1].voice.is_playing = true;
        simple_voice_stealer.mark_voice_as_active(&mut voices_with_state[1], 1);
        {
            let idle_voice = simple_voice_stealer.find_idle_voice(&mut voices_with_state, 0);
            assert_eq!(idle_voice.voice.index, 2);
        }
        voices_with_state[2].voice.is_playing = true;
        simple_voice_stealer.mark_voice_as_active(&mut voices_with_state[2], 2);
        {
            let idle_voice = simple_voice_stealer.find_idle_voice(&mut voices_with_state, 0);
            assert_eq!(idle_voice.voice.index, 0);
        }
    }

    #[test]
    fn test_simple_voice_stealer_find_voice_playing_note() {
        let number_of_voices = 3;
        let mut voices_with_state = vec![];
        let mut simple_voice_stealer = SimpleVoiceStealer::<TestVoice>::new();
        for i in 0..number_of_voices {
            voices_with_state.push(VoiceWithState {
                voice: TestVoice::new(i),
                state: SimpleVoiceStealerState::default()
            });
        }

        {
            let voice_playing = simple_voice_stealer.find_voice_playing_note(&mut voices_with_state, 0);
            assert_eq!(voice_playing, None);
        }
        voices_with_state[2].voice.is_playing = true;
        simple_voice_stealer.mark_voice_as_active(&mut voices_with_state[2], 2);
        {
            {
                let voice_playing = simple_voice_stealer.find_voice_playing_note(&mut voices_with_state, 2);
                match voice_playing {
                    None => unreachable!(),
                    Some(v) => assert_eq!(v.voice.index, 2)
                }
            }
            {
                let voice_idle = simple_voice_stealer.find_voice_playing_note(&mut voices_with_state, 1);
                assert_eq!(voice_idle, None);
            }
        }

        voices_with_state[1].voice.is_playing = true;
        simple_voice_stealer.mark_voice_as_active(&mut voices_with_state[1], 1);
        {
            {
                let voice_playing = simple_voice_stealer.find_voice_playing_note(&mut voices_with_state, 1);
                match voice_playing {
                    None => unreachable!(),
                    Some(v) => assert_eq!(v.voice.index, 1)
                }
            }
            {
                let voice_idle = simple_voice_stealer.find_voice_playing_note(&mut voices_with_state, 0);
                assert_eq!(voice_idle, None);
            }
        }

        voices_with_state[0].voice.is_playing = true;
        simple_voice_stealer.mark_voice_as_active(&mut voices_with_state[0], 0);
        {
            let voice_playing = simple_voice_stealer.find_voice_playing_note(&mut voices_with_state, 0);
            match voice_playing {
                None => unreachable!(),
                Some(v) => assert_eq!(v.voice.index, 0)
            }
        }
    }
}
