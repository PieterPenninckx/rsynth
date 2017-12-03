use asprim::AsPrim;
use num_traits::Float;
use note::{NoteData, NoteState};
use vst2::buffer::{Inputs, Outputs}; 
use envelope::Envelope;
use std::cell::Cell;


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
#[derive(Clone)]
pub struct Voice<T> where T: Renderable {
    /// The sample rate of the voice.  This is changed usually by the parent `Synth`
    pub sample_rate: Cell<f64>,
    /// Keeps track of what this voice is currently doing
    /// Unless this value is `VoiceState::Off`, the instrument
    /// will categorize this particular `Voice` as in-use
    pub state: VoiceState,
    /// Our own `Renderable` implementation
    pub sound: T,
    /// A number from -1 to 1 where 0 is center, and positive numbers are to the right
    pub pan: f64,
    /// Contains note data useful in determining what pitch to play.  This is used in tandem with the 
    /// `state` field.
    pub note_data: NoteData,
    /// The number of samples that have passed since the voice has begun playing
    sample_counter: f64,
    /// Contains the envelopes used for modifying various aspects of the `Voice`.
    pub envelopes: EnvelopeContainer,
    /// The current amplitude modifier, updated every sample
    pub amplitude_modifier: f64
}

impl<T> Voice<T> where T: Renderable {

    /// calls the voice's sound `render_next` function
    ///
    /// * `inputs` - a mutable reference to the input audio buffers 
    /// * `outputs` - a mutable reference to the output audio buffers to modify
    pub fn render_next<F: Float + AsPrim> (&mut self, inputs: &mut Inputs<F>, outputs: &mut Outputs<F>) {
        // temporary

        if self.note_data.state == NoteState::On {
            // calculate our amplitude envelope
            self.amplitude_modifier = self.envelopes.amplitude.interpolate(0f64);
            // render the user-defined audio stuff
            self.sound.render_next::<F, T>(inputs, outputs, self);
            // increment the samples (time) counter
            self.sample_counter += 1f64;
        } else {
            // TODO: release voice properly
            // reset the time counter
            self.sample_counter = 0f64;
            self.state = VoiceState::Off;
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

/// A struct that contains a variety of envelopes that our voice may need
#[derive(Clone)]
pub struct EnvelopeContainer {
    amplitude: Envelope
}

impl Default for EnvelopeContainer {
    fn default() -> Self {
        EnvelopeContainer {
            amplitude: Envelope::default()
        }
    }
}

pub struct VoiceBuilder<T> {
    /// The sample rate of the voice.  This is changed usually by the parent `Synth`
    sample_rate: Cell<f64>,
    /// Keeps track of what this voice is currently doing
    /// Unless this value is `VoiceState::Off`, the instrument
    /// will categorize this particular `Voice` as in-use
    state: VoiceState,
    /// Our own `Renderable` implementation
    sound: T,
    /// A number from -1 to 1 where 0 is center, and positive numbers are to the right
    pan: f64,
    /// Contains note data useful in determining what pitch to play.  This is used in tandem with the 
    /// `state` field.
    note_data: NoteData,
    /// The number of samples that have passed since the voice has begun playing
    sample_counter: f64,
    /// Contains the envelope used for modifying aspects of the voice.
    envelopes: EnvelopeContainer,
    /// The current amplitude modifier, updated every sample
    amplitude_modifier: f64
}

impl<T> VoiceBuilder<T> where T: Renderable {
    pub fn new_with_sound(sound: T) -> Self {
        VoiceBuilder {
            sample_rate: Cell::new(48_000f64),
            state: VoiceState::Off,
            sound: sound,
            pan: 0f64,
            note_data: NoteData::default(),
            sample_counter: 0f64,
            envelopes: EnvelopeContainer::default(),
            amplitude_modifier: 1f64
        }
    }

    pub fn sample_rate(mut self, sample_rate: f64) -> Self {
        self.sample_rate = Cell::new(sample_rate);
        self
    }

    pub fn amplitude_envelope(mut self, envelope: Envelope) -> Self {
        self.envelopes.amplitude = envelope;
        self
    }

    pub fn envelopes(mut self, envelopes: EnvelopeContainer) -> Self {
        self.envelopes = envelopes;
        self
    }

    pub fn finalize(mut self) -> Voice<T> {
        Voice {
            sample_rate: self.sample_rate,
            state: self.state,
            sound: self.sound,
            pan: self.pan,
            note_data: self.note_data,
            sample_counter: self.sample_counter,
            envelopes: self.envelopes,
            amplitude_modifier: self.amplitude_modifier
        }
    }
}

/// Keeps track of the current state of any voice
#[derive(PartialEq, Clone)]
pub enum VoiceState { 
    /// the voice is currently in use
    On,
    /// the voice has recieved a signal to stop and is now releasing 
    Releasing,
    /// the voice is not doing anything and can be used
    Off
}