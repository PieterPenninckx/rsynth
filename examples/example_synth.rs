// This file contains the actual sound generation of a plugin that is shared between all backends.
// The integration with Jack is in the `jack_synth.rs` file.

extern crate dasp_sample;
extern crate polyphony;

use polyphony::{
    midi::{RawMidiEventToneIdentifierDispatchClassifier, ToneIdentifier},
    simple_event_dispatching::{SimpleEventDispatcher, SimpleVoiceState},
    EventDispatchClassifier, Voice, VoiceAssigner,
};
use rsynth::event::{
    ContextualEventHandler, EventHandler, Indexed, RawMidiEvent, SysExEvent, Timed,
};
use rsynth::{AudioHandler, ContextualAudioRenderer};

use dasp_sample::{FromSample, Sample};
use midi_consts::channel_event::*;
use rsynth::backend::HostInterface;
use rsynth::buffer::AudioBufferInOut;
use rsynth::meta::{InOut, Meta, MetaData};
use std::f32::consts::PI;

static NUMBER_OF_VOICES: usize = 6;
static AMPLIFY_MULTIPLIER: f32 = 1.0 / NUMBER_OF_VOICES as f32;

// This struct defines the data that we will need to play one sine wave.
pub struct SineOscilator {
    // The step (how much we must proceed between two samples).
    frequency: f32,
    // The position (the number of which we are computing the sine wave.)
    position: f32,
    // The amplitude.
    amplitude: f32,
    // This is used to know if this is currently playing and if so, what note.
    state: SimpleVoiceState<ToneIdentifier>,
}

impl SineOscilator {
    fn new() -> Self {
        SineOscilator {
            frequency: 0.0,
            position: 0.0,
            amplitude: 0.0,
            state: SimpleVoiceState::Idle,
        }
    }

    fn get_sample(&mut self, frames_per_second: f32) -> f32 {
        // Note: this is a very naive implementation, just for demonstration purposes.
        if self.state == SimpleVoiceState::Idle {
            return 0.0;
        }
        let step = self.frequency / frames_per_second * 2.0 * PI;
        self.position += step;
        if self.position > 2.0 * PI {
            self.position -= 2.0 * PI;
        }
        self.position.sin() * self.amplitude
    }

    fn handle_event(&mut self, indexed: Indexed<Timed<RawMidiEvent>>) {
        let timed = dbg!(indexed.event);
        // We are digging into the details of midi-messages here.
        // Alternatively, you could use the `wmidi` crate.
        let data = timed.event.data();
        match (data[0] & EVENT_TYPE_MASK, data[1], data[2]) {
            (NOTE_OFF, _, _) | (NOTE_ON, _, 0) => {
                self.amplitude = 0.0;
                self.state = SimpleVoiceState::Idle;
            }
            (NOTE_ON, note_number, velocity) => {
                self.amplitude = dbg!(velocity as f32 / 127.0 * AMPLIFY_MULTIPLIER);
                self.frequency = dbg!(440.0 * 2.0_f32.powf(((note_number as f32) - 69.0) / 12.0));
                self.position = 0.0;
                self.state = SimpleVoiceState::Active(ToneIdentifier(timed.event.data()[1]));
            }
            _ => {}
        }
    }
}

// This enables using Sound in a polyphonic context.
impl Voice<SimpleVoiceState<ToneIdentifier>> for SineOscilator {
    fn state(&self) -> SimpleVoiceState<ToneIdentifier> {
        self.state
    }
}

pub struct SinePlayer {
    meta_data: MetaData<&'static str, &'static str, &'static str>,
    voices: Vec<SineOscilator>,
    sample_frequency: f32,
}

impl SinePlayer {
    fn meta_data() -> MetaData<&'static str, &'static str, &'static str> {
        MetaData {
            general_meta: "Simple sine player", // The name of the plugin
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
            voices.push(SineOscilator::new());
        }
        Self {
            meta_data: Self::meta_data(),
            voices: voices,
            sample_frequency: 44100.0,
        }
    }
}

impl Meta for SinePlayer {
    type MetaData = MetaData<&'static str, &'static str, &'static str>;

    fn meta(&self) -> &Self::MetaData {
        &self.meta_data
    }
}

impl AudioHandler for SinePlayer {
    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_frequency = sample_rate as f32;
    }
}

#[allow(unused_variables)]
impl<S, Context> ContextualAudioRenderer<S, Context> for SinePlayer
where
    S: FromSample<f32> + Sample,
    Context: HostInterface,
{
    fn render_buffer(&mut self, buffer: &mut AudioBufferInOut<S>, context: &mut Context) {
        if !context.output_initialized() {
            // Initialize the output buffer.
            buffer.outputs().set(S::EQUILIBRIUM);
        }
        let (left_channel, right_channel) = buffer.outputs().split_stereo();
        for (left, right) in left_channel.iter_mut().zip(right_channel.iter_mut()) {
            let mut sample = 0.0;
            for voice in self.voices.iter_mut() {
                sample += voice.get_sample(self.sample_frequency);
            }
            *left = S::from_sample_(sample);
            *right = *left;
        }
    }
}

impl<Context> ContextualEventHandler<Indexed<Timed<RawMidiEvent>>, Context> for SinePlayer {
    fn handle_event(&mut self, event: Indexed<Timed<RawMidiEvent>>, _context: &mut Context) {
        let classifier = RawMidiEventToneIdentifierDispatchClassifier;
        let classification = dbg!(classifier.classify(event.event.event.data()));
        let mut dispatcher = SimpleEventDispatcher;
        let assignment = dispatcher.assign(classification, &mut self.voices);
        assignment.dispatch(event, &mut self.voices, SineOscilator::handle_event);
    }
}

impl<'a, Context> ContextualEventHandler<Indexed<Timed<SysExEvent<'a>>>, Context> for SinePlayer {
    fn handle_event(&mut self, _event: Indexed<Timed<SysExEvent>>, _context: &mut Context) {
        // We don't do anything with SysEx events
    }
}
