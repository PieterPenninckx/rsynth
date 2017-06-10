use asprim::AsPrim;
use vst2::buffer::AudioBuffer;
use num_traits::Float;
use voice::Voice;
use voice::VoiceState;
use voice::Renderable;
/// The base structure for handling voices, sounds, and processing
/// You will always want to make this mutable.
///
/// * `voices` - A vector containing multiple objects implementing the `Voice` trait
/// * `sample_rate` - The sample rate the Synthesizer and voices should use
pub struct Synthesizer<T> where T: Renderable {
    
    /// A list of all voices a synthesizer contains.
    /// This is directly related to polyphony
    pub voices: Vec<Voice<T>>,
    pub sample_rate: f64,
    pub note_steal: StealMode,
}

impl<T> Default for Synthesizer<T> where T: Renderable{
    fn default () -> Self {
        Synthesizer { voices: vec![], sample_rate: 41_000f64, note_steal: StealMode::First }
    }
}


/// Contains all data needed to play a note
pub struct NoteData {
    /// An integer from 0-127 defining what note to play
    pub note: u8,
    /// An 8-bit unsigned value that can be used for modulating things such as amplitude
    pub velocity: u8,
    /// A float specifying pitch.  Use 0 for no change.
    pub pitch: f32,
    /// The On/Off state for a note
    pub state: NoteState
}

impl NoteData {
    /// return a default note.  This can be useful if you only care about one property
    fn new() -> NoteData{
        NoteData { note: 60u8, velocity: 127u8, pitch: 0f32, state: NoteState::On }
    }
}

/// A more readable boolean for keeping track of a note's state
pub enum NoteState {
    /// the note is on
    On,
    /// the note is off and should start `Releasing` a voice, if applicable
    Off
}

/// The way new notes will play if all voices are being currently utilized
/// This will change
pub enum StealMode {
    /// new notes will simply not be played if all voices are busy
    Off,
    /// stop playing the first voice to start playing in this frame
    First,
    /// stop playing the last voice to start playing in this frame
    Last
}

impl<T> Synthesizer<T> where T: Renderable {

    /// Begin playing with the specified note
    ///
    /// * `midi_note` - An integer from 0-127 defining what note to play
    /// * `velocty` - An 8-bit unsigned value that can be used for modulating things such as amplitude
    /// * `pitch` - A float specifying pitch.  Use 0 for no change.
    #[allow(unused_variables)]
    pub fn note_on(&self, note_data: NoteData){

        // Find a free voice and send this event
        for voice in &self.voices {

            match voice.state {
                VoiceState::On => { unimplemented!() },
                VoiceState::Releasing => { unimplemented!() },
                VoiceState::Off => {
                    voice.send_note(NoteData::new());
                    // we're done here!  Exit early.voice.state
                    return;
                }
            }

        }

        // note: this is most definitely not idiomatic rust and will need to be refactored.
        // We didn't find a free voice :( Steal one!
        match self.note_steal {
            StealMode::Off => { /* do nothing! */ },
            _ => {
                unimplemented!(); // TODO
            }
        }
    }

    /// Stop playing a specified note
    ///
    /// * `midi_note` - An integer from 0-127 defining what note to stop.  
    /// If this note is not currently "on", nothing will happen
    #[allow(unused_variables)]
    pub fn note_off(&self, midi_note: u8){
        unimplemented!()
    }

    /// Modify an audio buffer with rendered audio from the voice
    ///
    /// * `buffer` - the audio buffer to modify
    #[allow(unused_variables)]
    pub fn render_next<'a, F: Float + AsPrim>(&mut self, buffer: &AudioBuffer<'a, F>) {
        for voice in &mut self.voices {
            voice.render_next::<F>(buffer);
        }
    }
}

