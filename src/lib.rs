extern crate vst2;
extern crate asprim;
extern crate num_traits;

use self::asprim::AsPrim;
use self::vst2::buffer::AudioBuffer;
use self::num_traits::Float;
use self::voice::Voice;

pub mod voice;
pub mod sound;

/// The base structure for handling voices, sounds, and processing
/// You will always want to make this mutable.
///
/// * `voices` - A vector containing multiple objects implementing the `Voice` trait
/// * `sample_rate` - The sample rate the Synthesizer and voices should use
pub struct Synthesizer<T> where T: Voice {
    
    /// A list of all voices a synthesizer contains.
    /// This is directly related to polyphony
    pub voices: Vec<T>,
    pub sample_rate: f64,
    pub note_steal: StealMode
}

/// The way new notes will play if all voices are being currently utilized
pub enum StealMode {
    /// new notes will simply not be played if all voices are busy
    Off,
    /// stop playing the first voice to start playing in this frame
    First,
    /// stop playing the last voice to start playing in this frame
    Last,
    /// find the best voice to stop playing
    Smart
}

impl<T> Synthesizer<T> where T: Voice {

    /// Stop all notes from all `Voice`s 
    pub fn all_notes_off(&self){
        for voice in &self.voices {
            voice.note_off();
        }
    }

    /// Begin playing with the specified note
    ///
    /// * `midi_note` - An integer from 0-127 defining what note to play
    /// * `velocty` - An 8-bit unsigned value that can be used for modulating things such as amplitude
    /// * `pitch` - A float specifying pitch.  Use 0 for no change.
    pub fn note_on(&self, midi_note: u8, velocity: u8, pitch: f32){
        unimplemented!()
        // TODO: Find a free voice and send this event
    }

    /// Stop playing a specified note
    ///
    /// * `midi_note` - An integer from 0-127 defining what note to stop.  
    /// If this note is not currently "on", nothing will happen
    pub fn note_off(&self, midi_note: u8){
        unimplemented!()
    }

    /// Modify an audio buffer with rendered audio from the voice
    ///
    /// * `buffer` - the audio buffer reference to modify
    pub fn render_next<U: Float + AsPrim>(&self, buffer: &mut AudioBuffer<U>){
        unimplemented!()
        // TODO: render each voice in loop with some sort of way to combine
    }
}

