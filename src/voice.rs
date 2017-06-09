use asprim::AsPrim;
use num_traits::Float;
use synthesizer::NoteData;
use vst2::buffer::AudioBuffer; 


/// Implementing this on a struct will allow for custom audio processing
pub trait Renderable {

    /// Modify an audio buffer with rendered audio from the voice
    ///
    /// * `input` - the input audio buffer reference to modify
    /// * `output` - the output audio buffer reference to modify
    fn render_next<F: Float + AsPrim, T> (&mut self, inputs: Vec<&mut [F]>, outputs: Vec<&mut [F]>, voice: &Voice<T>) where T: Renderable;
}

/// A sampler / synthesizer voice. 
pub struct Voice<T> where T: Renderable {
    /// Keeps track of what this voice is currently doing
    pub state: VoiceState,
    /// A struct that defines how audio will render
    pub sound: T,
    /// a number from -1 to 1 where 0 is center
    pub panning: f32
}

impl<T> Voice<T> where T: Renderable {
    /// Controls the Voice based on note on/off signals
    ///
    /// * `note` - the `NoteData` to pass
    pub fn send_note(&self, note: NoteData){
        unimplemented!()
    }

    /// calls the voice's sound `render_next` function, passing in self for easy data access
    pub fn render_next<F: Float + AsPrim> (&mut self, buffer: &AudioBuffer<F>) {
        let (inputs, outputs) = buffer.split();
        &self.sound.render_next::<F, T>(inputs, outputs, self);
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