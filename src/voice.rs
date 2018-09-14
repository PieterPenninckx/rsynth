use asprim::AsPrim;
use envelope::Envelope;
use note::{NoteData, NoteState};
use num_traits::Float;
use std::cell::Cell;
use vst::buffer::{Inputs, Outputs};

/// Implementing this on a struct will allow for custom audio processing
pub trait Renderable {
    /// Modify an audio buffer with rendered audio from the voice
    ///
    /// * `inputs` - a mutable reference to the input audio buffers
    /// * `outputs` - a mutable reference to the output audio buffers to modify
    /// * `voice` - the `Voice` that conains this `Renderable` implementation.  This is useful
    /// if we need to access things like velocity in our DSP calculations
    fn render_next<F>(&mut self, inputs: &mut Inputs<F>, outputs: &mut Outputs<F>, voice_data: &VoiceData)
    where 
        F: Float + AsPrim;
}

/// An instrument voice.
#[derive(Clone)]
pub struct Voice<T>
where
    T: Renderable,
{
    /// Our own `Renderable` implementation
    pub renderable: T,
    /// Meta-data about this voice
    pub voice_data: VoiceData
}

#[derive(Clone)]
pub struct VoiceData
{
    /// The sample rate of the voice.  This is changed usually by the parent `Synth`
    pub sample_rate: Cell<f64>,
    /// Keeps track of what this voice is currently doing
    /// Unless this value is `VoiceState::Off`, the instrument
    /// will categorize this particular `Voice` as in-use
    pub state: VoiceState,
    /// A number from -1 to 1 where 0 is center, and positive numbers are to the right
    pub pan: f64,
    /// Contains note data useful in determining what pitch to play.  This is used in tandem with the
    /// `state` field.
    pub note_data: NoteData,
    /// Contains the envelopes used for modifying various aspects of the `Voice`.
    pub envelopes: EnvelopeContainer,
}

impl Default for VoiceData {
	fn default() -> Self {
		VoiceDataBuilder::default().finalize()
	}
}

impl<T> Voice<T>
where
    T: Renderable,
{
	pub fn new(voice_data: VoiceData, sound: T) -> Self {
		Voice {
			voice_data,
            renderable: sound
        }
	}
    /// calls the voice's sound `render_next` function
    ///
    /// * `inputs` - a mutable reference to the input audio buffers
    /// * `outputs` - a mutable reference to the output audio buffers to modify
    pub fn render_next<F: Float + AsPrim>(
        &mut self,
        inputs: &mut Inputs<F>,
        outputs: &mut Outputs<F>,
    ) {
        // temporary

        if self.voice_data.note_data.state == NoteState::On {
            // render the user-defined audio stuff
            self.renderable.render_next::<F>(inputs, outputs, &self.voice_data);
        } else {
            // TODO: release voice properly
            self.voice_data.state = VoiceState::Off;
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
    amplitude: Envelope,
}

impl Default for EnvelopeContainer {
    fn default() -> Self {
        EnvelopeContainer {
            amplitude: Envelope::default(),
        }
    }
}

pub struct VoiceDataBuilder {
    /// The sample rate of the voice.  This is changed usually by the parent `Synth`
    sample_rate: Cell<f64>,
    /// Keeps track of what this voice is currently doing
    /// Unless this value is `VoiceState::Off`, the instrument
    /// will categorize this particular `Voice` as in-use
    state: VoiceState,
    /// A number from -1 to 1 where 0 is center, and positive numbers are to the right
    pan: f64,
    /// Contains note data useful in determining what pitch to play.  This is used in tandem with the
    /// `state` field.
    note_data: NoteData,
    /// Contains the envelope used for modifying aspects of the voice.
    envelopes: EnvelopeContainer,
}

impl Default for VoiceDataBuilder {
	fn default() -> Self {
        VoiceDataBuilder {
            sample_rate: Cell::new(48_000f64),
            state: VoiceState::Off,
            pan: 0f64,
            note_data: NoteData::default(),
            envelopes: EnvelopeContainer::default(),
        }		
	}
}

impl VoiceDataBuilder {
    pub fn sample_rate(mut self, sample_rate: f64) -> Self {
        self.sample_rate = Cell::new(sample_rate);
        self
    }

    pub fn envelopes(mut self, envelopes: EnvelopeContainer) -> Self {
        self.envelopes = envelopes;
        self
    }

    pub fn finalize(self) -> VoiceData {
        VoiceData {
            sample_rate: self.sample_rate,
            state: self.state,
            pan: self.pan,
            note_data: self.note_data,
            envelopes: self.envelopes,
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
    Off,
}
