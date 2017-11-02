use asprim::AsPrim;
use vst2::buffer::AudioBuffer;
use vst2::api::Events;
use vst2::event::Event;
use num_traits::Float;
use voice::{Voice, VoiceState, Renderable};
use utility::*;
use utility::note::{NoteData, NoteState};

/// The base structure for handling voices, sounds, and processing
///
/// * `T` - a struct we create that implements the `Renderable` trait,
/// and contains all of our DSP code.
pub struct Synthesizer<T> where T: Renderable {
    
    /// A vector containing multiple objects implementing the `Voice` trait
    pub voices: Vec<Voice<T>>,
    /// A vector that keeps track of currently playing voices
    /// Each u8 refers to the note being played, and each 32b integer in the vector 
    /// corresponds to an index in `voices`.  Note that this data is duplicate of data
    /// in the `Voice` structure itself, so it shouldn't be relied upon for anything external,
    /// and must be manually updated when notes change.
    voices_used: Vec<(u8, usize)>,
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
    /// Note that although the framework supports any number of outputs,
    /// panning is currently only supported with stereo.
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
            pan_raw: (0f32, 0f32),
            voices_used: vec![]
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
            pan_raw: self.pan_raw,
            voices_used: vec![] }
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

        // split the buffer
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

    /// Process events from the plugin host.  This is useful if you are
    /// responding to MIDI notes and data.
    ///
    /// * `events` - a reference to an `Events` structure from the `vst2::api::Events`
    /// module. 
    pub fn process_events(&mut self, events: &Events) {
        // loop through all events
        for e in events.events() {
            // check if the event is a midi signal
            match e {
                Event::Midi(ev) => {
                    self.process_midi(NoteData::data(ev.data))
                },
                _ => return
            }
        }
    }

    /// Take in note data and turn a note on/off depending on the state
    fn process_midi(&mut self, note_data: NoteData){
        match note_data.state {
            NoteState::On => self.trigger_note_on(note_data),
            NoteState::Off => self.trigger_note_off(note_data),
            _ => return
        }
    }

    /// Used to find a voice to start playing.
    /// If voice stealing is enabled, it will take place here.
    fn trigger_note_on(&mut self, note_data: NoteData){
        // TODO: Voice stealing
        // for now, just find the first available voice
        // to keep mutability in our voice, use a simple mutable var i and increment in the loop
        // Here, `i` refers to the index of our `voices` vector.
        let mut i: usize = 0;

        for voice in &mut self.voices {
            if voice.state == VoiceState::Off {
                // Success.  Push our data to the vector containing "on" voices
                self.voices_used.push((note_data.note, i));
                // set our note data
                voice.note_data = note_data;
                voice.state = VoiceState::On;
                // exit early
                break;
            }
            // increment our iterator 
            i += 1;
        }
    }

    /// Finds a voice playing the same note as `note_data.note` and triggers that
    /// voice to begin releasing
    fn trigger_note_off(&mut self, note_data: NoteData){
        // index for `voices_used` 
        // HACK
        let mut remove_from_voices_used = false;
        let mut i = 0;

        // find the index of our voice in our `voices_used` array by the note number
        for &(note, voice_index) in &self.voices_used {
            if note == note_data.note {

                // Also assign the value `note_data`
                self.voices[voice_index].note_data = note_data;
                remove_from_voices_used = true;              
                break;
            }

            i += 1;
        }

        // remove index from the `voices_used` array and free it up for use again.
        if remove_from_voices_used {
            self.voices_used.remove(i);
        }
    }
}

/// An enum to display channel iterator numbers as readable data
pub enum Channel {
    Left,
    Right
}

/// Get a human readable `Channel` enum from a normal integer
fn channel_from_int(channel: usize) -> Channel {
    match channel {
        0 => Channel::Left,
        1 => Channel::Right,
        _ => Channel::Left
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