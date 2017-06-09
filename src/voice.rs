use asprim::AsPrim;
use vst2::buffer::AudioBuffer;
use num_traits::Float;
use synthesizer::NoteData;
use synthesizer::NoteState;


/// Implement this in order to allow a voice to be renderable
pub trait Renderable {

    /// Modify an audio buffer with rendered audio from the voice
    ///
    /// * `input` - the input audio buffer reference to modify
    /// * `output` - the output audio buffer reference to modify
    fn render_next<F: Float + AsPrim>(&mut self, input: &[F], output: &mut [F]);
}

/// A sampler / synthesizer voice. 
pub struct Voice<T> where T: Renderable {
    /// Keeps track of what this voice is currently doing
    pub state: VoiceState,
    /// A struct that defines how audio will render
    pub sound: T
}

impl<T> Voice<T> where T: Renderable {
    /// Controls the Voice based on note on/off signals
    ///
    /// * `note` - the `NoteData` to pass
    pub fn note(&self, note: NoteData){

    }
}

/// Keeps track of the current state of any voice
pub enum VoiceState { 
    /// the voice is currently in use
    On,
    /// the voice has recieved a signal to stop and is now releasing 
    Releasing,
    /// the voice is not doing anything and can be used
    Off
}