extern crate vst2;
extern crate asprim;
extern crate num_traits;

use self::asprim::AsPrim;
use self::vst2::buffer::AudioBuffer;
use self::num_traits::Float;
use synthesizer::NoteData;

/// Contains necessary methods for synth voices
pub trait Voice {

    /// Begin playing with the specified note
    ///
    /// * `midi_note` - An integer from 0-127 defining what note to play
    /// * `velocty` - An 8-bit unsigned value that can be used for modulating things such as amplitude
    /// * `pitch` - A float specifying pitch.  Use 0 for no change.
    fn note_on(&self, note_data: &NoteData);

    /// Stop playing a specified note
    fn note_off(&self);

    /// Modify an audio buffer with rendered audio from the voice
    ///
    /// * `buffer` - the audio buffer reference to modify
    fn render_next<T: Float + AsPrim>(&self, buffer: &mut AudioBuffer<T>);

    /// If the voice is currently playing something, return the note number
    /// Keep in mind that the `note_off` function is not always a reliable method
    /// to figure out if the Voice is playing due to ADSR.  Make sure to implement with that in mind.
    fn get_note(&self) ->  Option<u8>;
}