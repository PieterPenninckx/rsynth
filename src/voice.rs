use asprim::AsPrim;
use envelope::Envelope;
use note::{NoteData};
use num_traits::Float;
use backend::{InputAudioChannelGroup, OutputAudioChannelGroup};
use synth::SynthData;

/// Implementing this on a struct will allow for custom audio processing
pub trait Renderable {
    /// Modify an audio buffer with rendered audio from the voice
    ///
    /// * `inputs` - a mutable reference to the input audio buffers
    /// * `outputs` - a mutable reference to the output audio buffers to modify
    /// * `voice_data` - the `VoiceData` associated to the `Voice` that contains this `Renderable`
    ///     implementation. This is useful if we need to access things like velocity
    ///     in our DSP calculations
    fn render_next<'a, F, In, Out>(
            &mut self, inputs: &In,
            outputs: &'a mut Out,
            voice_data: &VoiceData,
            synth_data: &SynthData,
    )
    where 
        F: Float + AsPrim,
        In: InputAudioChannelGroup<F>,
        Out: OutputAudioChannelGroup<F>,
        &'a mut Out: IntoIterator<Item = &'a mut[F]>;
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
    pub fn render_next<'a, F, In, Out>(
        &mut self,
        inputs: &In,
        outputs: &'a mut Out,
        synth_data: &SynthData
    )
    where
        F: Float + AsPrim,
        In: InputAudioChannelGroup<F>,
        Out: OutputAudioChannelGroup<F>,
        &'a mut Out: IntoIterator<Item = &'a mut [F]>
    {
        if self.voice_data.state != VoiceState::Off {
            // render the user-defined audio stuff
            self.renderable.render_next::<F, _, _>(inputs, outputs, &self.voice_data, synth_data);
        }
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
            state: VoiceState::Off,
            pan: 0f64,
            note_data: NoteData::default(),
            envelopes: EnvelopeContainer::default(),
        }		
	}
}

impl VoiceDataBuilder {


    pub fn envelopes(mut self, envelopes: EnvelopeContainer) -> Self {
        self.envelopes = envelopes;
        self
    }

    pub fn finalize(self) -> VoiceData {
        VoiceData {
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
    /// the voice has received a signal to stop and is now releasing
    Releasing,
    /// the voice is not doing anything and can be used
    Off,
}
