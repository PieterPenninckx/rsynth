// This file contains the actual sound generation of a plugin that is shared between all backends.
// The integration with VST is in the `vst_synt.rs` file.
// The integration with Jack is in the `jack_synth.rs` file.
use asprim::AsPrim;
use num_traits::Float;
use rand::{thread_rng, Rng};
use rsynth::event::{ContextualEventHandler, EventHandler, RawMidiEvent, SysExEvent, Timed};
use rsynth::middleware::polyphony::{
    voice_stealer::{AssignFirstIdleVoice, BasicState},
    ToneIdentifier, Voice, VoiceStealer,
};
use rsynth::{AudioRendererMeta, CommonAudioPortMeta, CommonPluginMeta, ContextualAudioRenderer};

use rsynth::event::raw_midi_event_event_types::*;

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
    state: BasicState<ToneIdentifier>,
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
            state: BasicState::Idle,
        }
    }

    // Here, we use one implementation over all floating point types.
    // If you want to use SIMD optimization, you can have separate implementations
    // for `f32` and `f64`.
    fn render_audio_buffer<F>(&mut self, outputs: &mut [&mut [F]])
    where
        F: AsPrim + Float,
    {
        if self.state == BasicState::Idle {
            return;
        }
        assert_eq!(2, outputs.len());
        // for every output
        for output in outputs {
            // for each value in the buffer
            for sample in output.iter_mut() {
                // We "add" to the output.
                // In this way, various noises can be heard together.
                *sample =
                    *sample + self.white_noise[self.position].as_::<F>() * self.amplitude.as_();
                // Increment the position of our sound sample.
                // We loop this easily by using modulo.
                self.position = (self.position + 1) % self.white_noise.len();
            }
        }
    }
}

// This enables using Sound in a polyphonic context.
impl Voice<BasicState<ToneIdentifier>> for Noise {
    fn state(&self) -> BasicState<ToneIdentifier> {
        self.state
    }
}

impl EventHandler<Timed<RawMidiEvent>> for Noise {
    fn handle_event(&mut self, timed: Timed<RawMidiEvent>) {
        let state_and_chanel = timed.event.data()[0];

        // We are digging into the details of midi-messages here.
        // Alternatively, you could use the `wmidi` crate.
        if state_and_chanel & RAW_MIDI_EVENT_EVENT_TYPE_MASK == RAW_MIDI_EVENT_NOTE_ON {
            self.amplitude = timed.event.data()[2] as f32 / 127.0 * AMPLIFY_MULTIPLIER;
            self.state = BasicState::Active(ToneIdentifier(timed.event.data()[1]));
        }
        if state_and_chanel & RAW_MIDI_EVENT_EVENT_TYPE_MASK == RAW_MIDI_EVENT_NOTE_OFF {
            self.amplitude = 0.0;
            self.state = BasicState::Idle;
        }
    }
}

pub struct NoisePlayer {
    voices: Vec<Noise>,
    dispatcher: AssignFirstIdleVoice<ToneIdentifier>,
}

impl NoisePlayer {
    pub fn new() -> Self {
        let mut voices = Vec::new();
        for _ in 0..NUMBER_OF_VOICES {
            voices.push(Noise::new(SAMPLE_SIZE));
        }
        Self {
            voices: voices,
            dispatcher: AssignFirstIdleVoice::new(),
        }
    }
}

impl AudioRendererMeta for NoisePlayer {
    // We have no audio inputs:
    const MAX_NUMBER_OF_AUDIO_INPUTS: usize = 0;
    // We expect stereo output:
    const MAX_NUMBER_OF_AUDIO_OUTPUTS: usize = 2;

    fn set_sample_rate(&mut self, sample_rate: f64) {
        trace!("set_sample_rate(sample_rate={})", sample_rate);
        // We are not doing anything with this right now.
    }
}

impl CommonPluginMeta for NoisePlayer {
    // This is the name of our plugin.
    const NAME: &'static str = "RSynth Example";
}

impl CommonAudioPortMeta for NoisePlayer {
    fn audio_output_name(index: usize) -> String {
        trace!("audio_output_name(index = {})", index);
        match index {
            0 => "left".to_string(),
            1 => "right".to_string(),
            _ => {
                "".to_string()
                // We have specified that we only support two output channels,
                // so the host should not try to get the name of the third output
                // channel.
                // If we get at this point, this would indicate a bug in the host
                // because we have only specified two audio outputs.
            }
        }
    }
}

#[allow(unused_variables)]
impl<F, Context> ContextualAudioRenderer<F, Context> for NoisePlayer
where
    F: AsPrim + Float,
{
    fn render_buffer(
        &mut self,
        _inputs: &[&[F]],
        outputs: &mut [&mut [F]],
        _context: &mut Context,
    ) {
        for noise in self.voices.iter_mut() {
            noise.render_audio_buffer(outputs);
        }
    }
}

impl<Context> ContextualEventHandler<Timed<RawMidiEvent>, Context> for NoisePlayer {
    fn handle_event(&mut self, event: Timed<RawMidiEvent>, _context: &mut Context) {
        self.dispatcher.dispatch_event(event, &mut self.voices)
    }
}

impl<'a, Context> ContextualEventHandler<Timed<SysExEvent<'a>>, Context> for NoisePlayer {
    fn handle_event(&mut self, _event: Timed<SysExEvent<'a>>, _context: &mut Context) {
        // We don't do anything with SysEx events.
    }
}
