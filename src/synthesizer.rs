use asprim::AsPrim;
use vst2::buffer::AudioBuffer;
use num_traits::Float;
use voice::{Voice, VoiceState, Renderable};
use utility::*;

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
    pub steal_mode: StealMode,
    /// The entire balance of the instrument
    pan: f32,
    /// The raw amp values for panning
    /// Only modify these if you know what you're doing
    pub pan_raw: (f32, f32)
}

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
    pub fn voices(mut self, voices: Vec<Voice<T>>) -> Self {
        self.voices = voices;
        self
    }

    /// Set the sample rate using the builder
    pub fn sample_rate(mut self, sample_rate: f64) -> Self {
        self.sample_rate = sample_rate;
        self
    }

    /// Set the note steal mode using the builder
    pub fn steal_mode(mut self, steal_mode: StealMode) -> Self {
        self.steal_mode = steal_mode;
        self
    }

    /// Finalize the builder
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
        match self.steal_mode {
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

        // Do some more generic processing on the sound for basic functionality
        // This happens synth-wide, not per-voice.
        // WARNING: This essentially loops twice when it isn't needed
        // This will be changed in the future, most likely
        for (i, output) in outputs.into_iter().enumerate() {

            // Process
            self.post_process(output, i);
        }
    }

    /// Process the entire instrument
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

}


/// An enum to display channel numbers as readable data
pub enum Channel {
    Left,
    Right
}

#[allow(match_same_arms)]
fn channel_from_int(channel: usize) -> Channel {
    match channel {
        0 => Channel::Left,
        1 => Channel::Right,
        _ => Channel::Left
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
    pub state: NoteState,
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