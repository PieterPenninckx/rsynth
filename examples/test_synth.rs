// This file contains the actual sound generation of a plugin that is shared between all backends.
// The integration with VST is in the `vst_synt.rs` file.
// The integration with Jack is in the `jack_synth.rs` file.

extern crate polyphony;

use asprim::AsPrim;
use num_traits::Float;
use rand::{thread_rng, Rng};
use rsynth::event::{
    ContextualEventHandler, EventHandler, Indexed, RawMidiEvent, SysExEvent, Timed,
};
use polyphony::{
    simple_event_dispatching::{SimpleEventDispatcher, SimpleVoiceState},
    Voice, EventDispatchClassifier, VoiceAssigner,
    midi::{ToneIdentifier, RawMidiEventToneIdentifierDispatchClassifier}
};
use rsynth::{AudioHandler, ContextualAudioRenderer};

use midi_consts::channel_event::*;
use rsynth::buffer::AudioBufferInOut;
use rsynth::meta::{InOut, Meta, MetaData};

// The total number of samples to pre-calculate.
// This is like recording a sample of white noise and then
// using it as an oscillator.  It saves on CPU overhead by
// preventing us from having to use a random function each sample.
static SAMPLE_SIZE: usize = 65536;
static NUMBER_OF_VOICES: usize = 6;
static AMPLIFY_MULTIPLIER: f32 = 1.0 / NUMBER_OF_VOICES as f32;

// This struct defines the data that we will need to play one "noise"
pub struct Noise {
    // Random data of the noise.
    white_noise: Vec<f32>,
    // At which sample in the noise we are.
    position: usize,
    // The amplitude.
    amplitude: f32,
    // This is used to know if this is currently playing and if so, what note.
    state: SimpleVoiceState<ToneIdentifier>,
}

impl Noise {
    fn new(sample_size: usize) -> Self {
        let mut rng = thread_rng();
        let samples: Vec<f32> = rng
            .gen_iter::<f32>()
            .take(sample_size)
            .map(|r| {
                // The random generator generates noise between 0 and 1,
                // we map it to the range -1 to 1.
                2.0 * r - 1.0
            })
            .collect::<Vec<f32>>();
        Noise {
            white_noise: samples,
            position: 0,
            amplitude: 0.0,
            state: SimpleVoiceState::Idle,
        }
    }

    // Here, we use one implementation over all floating point types.
    // If you want to use SIMD optimization, you can have separate implementations
    // for `f32` and `f64`.
    fn render_audio_buffer<S>(&mut self, buffer: &mut AudioBufferInOut<S>)
    where
        S: AsPrim + Float,
    {
        if self.state == SimpleVoiceState::Idle {
            return;
        }
        let outputs = buffer.outputs();
        assert_eq!(2, outputs.number_of_channels());
        for output_channel in outputs.channel_iter_mut() {
            for sample in output_channel.iter_mut() {
                // We "add" to the output.
                // In this way, various noises can be heard together.
                *sample =
                    *sample + self.white_noise[self.position].as_::<S>() * self.amplitude.as_();
                // Increment the position of our sound sample.
                // We loop this easily by using modulo.
                self.position = (self.position + 1) % self.white_noise.len();
            }
        }
    }
}

// This enables using Sound in a polyphonic context.
impl Voice<SimpleVoiceState<ToneIdentifier>> for Noise {
    fn state(&self) -> SimpleVoiceState<ToneIdentifier> {
        self.state
    }
}

impl EventHandler<Timed<RawMidiEvent>> for Noise {
    fn handle_event(&mut self, timed: Timed<RawMidiEvent>) {
        let state_and_chanel = timed.event.data()[0];

        // We are digging into the details of midi-messages here.
        // Alternatively, you could use the `wmidi` crate.
        if state_and_chanel & EVENT_TYPE_MASK == NOTE_ON {
            self.amplitude = timed.event.data()[2] as f32 / 127.0 * AMPLIFY_MULTIPLIER;
            self.state = SimpleVoiceState::Active(ToneIdentifier(timed.event.data()[1]));
        }
        if state_and_chanel & EVENT_TYPE_MASK == NOTE_OFF {
            self.amplitude = 0.0;
            self.state = SimpleVoiceState::Idle;
        }
    }
}

pub struct NoisePlayer {
    meta_data: MetaData<&'static str, &'static str, &'static str>,
    voices: Vec<Noise>,
}

impl NoisePlayer {
    fn meta_data() -> MetaData<&'static str, &'static str, &'static str> {
        MetaData {
            general_meta: "Noise generator", // The name of the plugin
            audio_port_meta: InOut {
                inputs: Vec::new(),             // No audio inputs
                outputs: vec!["left", "right"], // Two audio outputs
            },
            midi_port_meta: InOut {
                inputs: vec!["midi in"], // One midi in port
                outputs: Vec::new(),     // No midi out port
            },
        }
    }
    pub fn new() -> Self {
        let mut voices = Vec::new();
        for _ in 0..NUMBER_OF_VOICES {
            voices.push(Noise::new(SAMPLE_SIZE));
        }
        Self {
            meta_data: Self::meta_data(),
            voices: voices,
        }
    }
}

impl Meta for NoisePlayer {
    type MetaData = MetaData<&'static str, &'static str, &'static str>;

    fn meta(&self) -> &Self::MetaData {
        &self.meta_data
    }
}

impl AudioHandler for NoisePlayer {
    fn set_sample_rate(&mut self, sample_rate: f64) {
        trace!("set_sample_rate(sample_rate={})", sample_rate);
        // We are not doing anything with this right now.
    }
}

#[allow(unused_variables)]
impl<S, Context> ContextualAudioRenderer<S, Context> for NoisePlayer
where
    S: AsPrim + Float,
{
    fn render_buffer(&mut self, buffer: &mut AudioBufferInOut<S>, _context: &mut Context) {
        for noise in self.voices.iter_mut() {
            noise.render_audio_buffer(buffer);
        }
    }
}

impl<Context> ContextualEventHandler<Timed<RawMidiEvent>, Context> for NoisePlayer {
    fn handle_event(&mut self, event: Timed<RawMidiEvent>, _context: &mut Context) {
        let classifier = RawMidiEventToneIdentifierDispatchClassifier;
        let classification = classifier.classify(event.event.data());
        let mut dispatcher = SimpleEventDispatcher;
        let assignment = dispatcher.assign(classification, &mut self.voices);
        assignment.dispatch(event, &mut self.voices, Noise::handle_event);
    }
}

// Only needed for Jack: delegate to the normal event handler.
impl<Context> ContextualEventHandler<Indexed<Timed<RawMidiEvent>>, Context> for NoisePlayer {
    fn handle_event(&mut self, event: Indexed<Timed<RawMidiEvent>>, context: &mut Context) {
        self.handle_event(event.event, context)
    }
}

impl<'a, Context> ContextualEventHandler<Timed<SysExEvent<'a>>, Context> for NoisePlayer {
    fn handle_event(&mut self, _event: Timed<SysExEvent<'a>>, _context: &mut Context) {
        // We don't do anything with SysEx events.
    }
}

// Only needed for Jack: delegate to the normal event handler.
impl<'a, Context> ContextualEventHandler<Indexed<Timed<SysExEvent<'a>>>, Context> for NoisePlayer {
    fn handle_event(&mut self, event: Indexed<Timed<SysExEvent>>, context: &mut Context) {
        self.handle_event(event.event, context)
    }
}
