use asprim::AsPrim;
use num_traits::Float;
use utility::*;
use vst2::buffer::{Inputs, Outputs}; 

/// Implementing this on a struct will allow for custom audio processing
pub trait Renderable {

    /// Modify an audio buffer with rendered audio from the voice
    ///
    /// * `inputs` - a mutable reference to the input audio buffers 
    /// * `outputs` - a mutable reference to the output audio buffers to modify
    /// * `voice` - the `Voice` that conains this `Renderable` implementation.  This is useful
    /// if we need to access things like velocity in our DSP calculations
    fn render_next<F, T> (&self, inputs: &mut Inputs<F>, outputs: &mut Outputs<F>, voice: &Voice<T>)
        where T: Renderable,
              F: Float + AsPrim;
}

/// An instrument voice.
pub struct Voice<T> where T: Renderable {
    /// Keeps track of what this voice is currently doing
    /// Unless this value is `VoiceState::Off`, the instrument
    /// will categorize this particular `Voice` as in-use
    pub state: VoiceState,
    /// Our own `Renderable` implementation
    pub sound: T,
    /// A number from -1 to 1 where 0 is center, and positive numbers are to the right
    pub pan: f32,
    /// Contains note data useful in determining what pitch to play.  This is used in tandem with the 
    /// `state` field.
    pub note: NoteData
}

impl<T> Voice<T> where T: Renderable {

    /// calls the voice's sound `render_next` function
    ///
    /// * `inputs` - a mutable reference to the input audio buffers 
    /// * `outputs` - a mutable reference to the output audio buffers to modify
    pub fn render_next<F: Float + AsPrim> (&self, inputs: &mut Inputs<F>, outputs: &mut Outputs<F>) {
        
        // temporary
        if self.note.state == NoteState::On {
            self.sound.render_next::<F, T>(inputs, outputs, self);
        }

        /*
        // determine how to play the sound based on the statue of our voice
        match self.state {
            VoiceState::Off => { },
            _ => {
                // Send the buffer to our sound implementation for processing
                self.sound.render_next::<F, T>(inputs, outputs, self);
            }
        }
        */
    }
}


/// Keeps track of the current state of any voice
#[derive(PartialEq)]
pub enum VoiceState { 
    /// the voice is currently in use
    On,
    /// the voice has recieved a signal to stop and is now releasing 
    Releasing,
    /// the voice is not doing anything and can be used
    Off
}