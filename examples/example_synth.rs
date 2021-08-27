// This file contains the actual sound generation of a plugin that is shared between all backends.
// The integration with Jack is in the `jack_synth.rs` file.

extern crate dasp_sample;
extern crate polyphony;

use polyphony::{
    midi::{RawMidiEventToneIdentifierDispatchClassifier, ToneIdentifier},
    simple_event_dispatching::{SimpleEventDispatcher, SimpleVoiceState},
    EventDispatchClassifier, Voice, VoiceAssigner,
};
#[cfg(feature = "backend-jack")]
use rsynth::derive_jack_port_builder;
use rsynth::event::{
    CoIterator, ContextualEventHandler, EventHandler, Indexed, RawMidiEvent, SysExEvent, Timed,
};
use rsynth::{derive_ports, ContextualAudioRenderer};

use dasp_sample::{FromSample, Sample};
use midi_consts::channel_event::*;
use rsynth::backend::HostInterface;
use rsynth::meta::{InOut, Meta, MetaData};
use rsynth::AudioHandler;
use std::f32::consts::PI;

static NUMBER_OF_VOICES: usize = 6;
static AMPLIFY_MULTIPLIER: f32 = 1.0 / NUMBER_OF_VOICES as f32;

/*
trace_macros!(true);
derive_ports! {
    struct SineOscilatorPorts<'a> {
        audio_in: &'a [f32],
        audio_out: &'a mut [f32],
        midi_in: &'a mut dyn Iterator<Item = Timed<RawMidiEvent>>,
    }


    derive_jack_port_builder! {
        struct SineOscilatorPortsBuilder {
            generate_fields!();
        }
    }
}
trace_macros!(false);

 */

derive_ports! {
    struct SineOscilatorPorts<'a> {
        out_left: &'a mut [f32],
        out_right: &'a mut [f32],
        midi_in: &'a mut dyn Iterator<Item = Timed<RawMidiEvent>>,
    }

    derive_jack_port_builder! {
        struct SineOscilatorPortsBuilder {
            generate_fields!()
        }
    }
}

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
    pub fn new() -> Self {
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
        let timed = indexed.event;
        // We are digging into the details of midi-messages here.
        // Alternatively, you could use the `wmidi` crate.
        let data = timed.event.data();
        match (data[0] & EVENT_TYPE_MASK, data[1], data[2]) {
            (NOTE_OFF, _, _) | (NOTE_ON, _, 0) => {
                self.amplitude = 0.0;
                self.state = SimpleVoiceState::Idle;
            }
            (NOTE_ON, note_number, velocity) => {
                self.amplitude = velocity as f32 / 127.0 * AMPLIFY_MULTIPLIER;
                self.frequency = 440.0 * 2.0_f32.powf(((note_number as f32) - 69.0) / 12.0);
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

impl<'a, Context> ContextualAudioRenderer<SineOscilatorPorts<'a>, Context> for SinePlayer
where
    Context: HostInterface,
{
    fn render_buffer(&mut self, ports: SineOscilatorPorts<'a>, context: &mut Context) {
        for (left, right) in ports.out_left.iter_mut().zip(ports.out_right.iter_mut()) {
            let mut sample = 0.0;
            for voice in self.voices.iter_mut() {
                sample += voice.get_sample(self.sample_frequency);
            }
            *left = sample;
            *right = sample;
        }
    }
}
