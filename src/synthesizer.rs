use asprim::AsPrim;
use vst2::buffer::AudioBuffer;
use vst2::api::Events;
use vst2::event::Event;
use num_traits::Float;
use voice::{Voice, VoiceState, Renderable};
use utility::*;

/// The base structure for handling voices, sounds, and processing
///
/// * `T` - a struct we create that implements the `Renderable` trait,
/// and contains all of our DSP code.
pub struct Synthesizer<T> where T: Renderable {
    
    /// A vector containing multiple objects implementing the `Voice` trait
    pub voices: Vec<Voice<T>>,
    /// The sample rate the Synthesizer and voices should use
    pub sample_rate: f64,
    /// What method the synth should use to steal voices (if any)
    pub steal_mode: StealMode,
    /// The balance of the instrument represented as a float between -1 and 1, 
    /// where 0 is center and 1 is to the right.
    pan: f32,
    /// The raw amp values for panning
    /// This can be used in tandem with a state object to set the global
    /// panning values every block render, without having to perform
    /// an expensive panning formula every time.  For instance, we can
    /// calculate `constant_power_pan` in a callback every time the pan knob is moved
    /// and assign that value to a tuple.
    /// Then, before calling the `render_next` method on our synth, we can set the
    /// `pan_raw` field to our aforementioned tuple. 
    pub pan_raw: (f32, f32)
}

/// Get default values 
/// This is only really useful with our internal builder methods.
/// If we try something like `let s = { sample_rate: 48_000, .. Synthesizer::default() };`
/// the compiler will complain that some fields are private.
impl<T> Default for Synthesizer<T> where T: Renderable{
    fn default () -> Self {
        Synthesizer { 
            voices: vec![], 
            sample_rate: 41_000f64, 
            steal_mode: StealMode::First, 
            pan: 0f32,
            pan_raw: (0f32, 0f32)
        }
    }
}

impl<T> Synthesizer<T> where T: Renderable {

    /// Constructor for the Synthesizer utilizing a builder pattern
    pub fn new() -> Self {
        Synthesizer::default()
    }

    /// Set voices using the builder
    ///
    /// * `voices` - A vector containing any number of `Voice` structures.
    /// If our instrument is polyphonic, the number of voices will determine the maximum amount
    /// of notes it can play at once.
    pub fn voices(mut self, voices: Vec<Voice<T>>) -> Self {
        self.voices = voices;
        self
    }

    /// Set the sample rate using the builder
    ///
    /// * `sample_rate` - set the sample rate of our instrument
    pub fn sample_rate(mut self, sample_rate: f64) -> Self {
        self.sample_rate = sample_rate;
        self
    }

    /// Set the note steal mode using the builder
    ///
    /// * `steal_mode` - this determines how "voice stealing" will be implemented, if at all.
    pub fn steal_mode(mut self, steal_mode: StealMode) -> Self {
        self.steal_mode = steal_mode;
        self
    }

    /// Finalize the builder and return an immutable `Synthesizer`
    #[allow(unused_variables)]
    pub fn finalize(self) -> Self {
        let (pan_left_amp, pan_right_amp) = constant_power_pan(self.pan);
        Synthesizer { 
            pan: self.pan, 
            voices: self.voices, 
            sample_rate: self.sample_rate, 
            steal_mode: self.steal_mode,
            pan_raw: self.pan_raw }
    }

    /// Send midi note data to a voice
    ///
    /// * `note_data` - contains all information needed to play a note
    #[allow(unused_variables)]
    pub fn set_voice_note(&mut self, note_data: NoteData){

        // Find a free voice and send this event
        for voice in &mut self.voices {

            // for now, just set it without checking
            voice.note = note_data;

            match voice.state {
                VoiceState::On => { unimplemented!() },
                VoiceState::Releasing => { unimplemented!() },
                VoiceState::Off => {
                    // send a note
                    // we're done here!  Exit early.voice.state
                    return;
                }
            }

        }
    }

    /// Set the panning for the entire instrument
    /// This is done via a function instead of directly setting the field
    /// as the formula is potentially costly and should only be calculated
    /// when needed.  For instance, do not use this function in a loop for
    /// every sample.  Instead, update the value only when parameters change.
    /// If you need to set the panning every block render, consider accessing
    /// the `pan_raw` field directly.
    ///
    /// * `amount` - a float value between -1 and 1 where 0 is center and 1 is to the right.
    /// Values not within this range will be 
    pub fn set_pan(&mut self, amount: f32){
        self.pan = amount;
        let (pan_left_amp, pan_right_amp) = constant_power_pan(self.pan);
        self.pan_raw = (pan_left_amp, pan_right_amp);
    }

    /// Modify an audio buffer with rendered audio from the voice
    ///
    /// * `buffer` - the audio buffer to modify
    #[allow(unused_variables)]
    pub fn render_next<'a, F: Float + AsPrim>(&mut self, buffer: &mut AudioBuffer<'a, F>) {

        /// split the buffer
        let (mut inputs, mut outputs) = buffer.split();
        for voice in &mut self.voices {
            voice.render_next::<F>(&mut inputs, &mut outputs);
        }

        /// Do some more generic processing on the sound for basic functionality
        /// This happens synth-wide, not per-voice.
        /// WARNING: This essentially loops twice when it isn't needed
        /// This will be changed in the future, most likely
        for (i, output) in outputs.into_iter().enumerate() {

            // Process
            self.post_process(output, i);
        }
    }

    

    /// Process the entire instrument through generic effects like instrument-wide panning and volume
    ///
    /// * `output` - a mutable reference to a single output buffer
    /// * `channel_i` - the iterator number that relates to the `output` index.  This determines
    /// what channel the method is currently processing.  For example, `0 == Channel::Left` and
    /// `1 == Channel::Right`. 
    fn post_process<F: Float + AsPrim>(&self, output: &mut [F], channel_i: usize) {
        let channel = channel_from_int(channel_i);

        for sample in output {

            // Do channel specific stuff first
            match channel {
                Channel::Left => {
                    *sample = *sample * self.pan_raw.0.as_();
                }
                Channel::Right => {
                    *sample = *sample * self.pan_raw.1.as_();
                }
            }
        }
    }

    /// 
    #[allow(match_same_arms)]
    pub fn process_events(&mut self, events: &Events) {
        // loop through all events
        for e in events.events() {
            // check if the event is a midi signal
            match e {
                Event::Midi { data, .. } => {

                    // extract our data and call the appropriate function
                    let note_data = NoteData::data(data); 
                    self.set_voice_note(note_data);
                    return
                }
                _ => return
            }
        }
    }

}


/// An enum to display channel iterator numbers as readable data
pub enum Channel {
    Left,
    Right
}

#[allow(match_same_arms)]
/// Get a human readable `Channel` enum from a normal integer
fn channel_from_int(channel: usize) -> Channel {
    match channel {
        0 => Channel::Left,
        1 => Channel::Right,
        _ => Channel::Left
    }
}

impl Default for NoteData {
    fn default() -> NoteData {
        NoteData { note: 60u8, velocity: 127u8, state: NoteState::Nil, channel: 0 }
    }
}

/// Builder methods
impl NoteData {
    pub fn new() -> Self {
        NoteData::default()
    }

    pub fn note(&mut self, note: u8) -> &mut Self {
        self.note = note;
        self
    }

    pub fn velocity(&mut self, velocity: u8) -> &mut Self {
        self.velocity = velocity;
        self
    }

    pub fn channel(&mut self, channel: u8) -> &mut Self {
        self.channel = channel;
        self
    }

    pub fn state(&mut self, state: NoteState) -> &mut Self {
        self.state = state;
        self
    }

    pub fn finalize(self) -> Self {
        NoteData { note: self.note, velocity: self.velocity, channel: self.channel, state: self.state }
    }
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