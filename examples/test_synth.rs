// This file contains the actual sound generation of a plugin that is shared between all backends.
// The integration with VST is in the `vst_synt.rs` file.
// The integration with Jack is in the `jack_synth.rs` file.
use asprim::AsPrim;
use num_traits::Float;
use rand::{thread_rng, Rng};
use rsynth::backend::HostInterface;
use rsynth::event::{EventHandler, RawMidiEvent, SysExEvent, Timed};
use rsynth::middleware::polyphony::Voice;
use rsynth::output_mode::OutputMode;
use rsynth::Plugin;
use std::env;
use std::fs::File;

use simplelog::*;

// The total number of samples to pre-calculate
// This is like recording a sample of white noise and then
// using it as an oscillator.  It saves on CPU overhead by
// preventing us from having to use a random function each sample.
static SAMPLE_SIZE: usize = 65536;
static AMPLIFY_MULTIPLIER: f32 = 0.2;

#[derive(Clone)]
pub struct Sound<M>
where
    M: OutputMode,
{
    white_noise: Vec<f32>,
    sample_count: usize,
    position: usize,
    velocity: u8,
    is_playing: bool,
    mode: M,
}

impl<M> Default for Sound<M>
where
    M: OutputMode,
{
    fn default() -> Self {
        // You can use the `log` crate for debugging purposes.
        trace!("default()");

        let mut rng = thread_rng();
        let samples: Vec<f32> = rng
            .gen_iter::<f32>()
            .take(SAMPLE_SIZE)
            .collect::<Vec<f32>>();
        Sound {
            sample_count: samples.len(),
            white_noise: samples,
            position: 0,
            velocity: 0,
            is_playing: false,
            mode: M::default(),
        }
    }
}

/// The DSP stuff goes here
impl<M, H> Plugin<H> for Sound<M>
where
    M: OutputMode,
    H: HostInterface,
{
    // This is the name of our plugin.
    const NAME: &'static str = "RSynth Example";

    // We have no audio inputs:
    const MAX_NUMBER_OF_AUDIO_INPUTS: usize = 0;

    // We expect stereo output:
    const MAX_NUMBER_OF_AUDIO_OUTPUTS: usize = 2;

    fn audio_input_name(index: usize) -> String {
        trace!("audio_input_name(index = {})", index);

        // Because we have specified that our plugin has no audio input,
        // the `audio_input_name` function should not be called by the host.
        // So we can just return an empty string here.
        "".to_string()
    }

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

    fn set_sample_rate(&mut self, sample_rate: f64) {
        trace!("set_sample_rate(sample_rate={})", sample_rate);
        // We are not doing anything with this right now.
    }

    #[allow(unused_variables)]
    fn render_buffer<F>(&mut self, inputs: &[&[F]], outputs: &mut [&mut [F]], _context: &mut H)
    where
        F: Float + AsPrim,
    {
        assert_eq!(2, outputs.len());
        // for every output
        for output in outputs {
            // for each value in buffer
            for (i, sample) in output.iter_mut().enumerate() {
                // Increment the position of our sound sample.
                // We loop this easily by using modulo.
                self.position = (self.position + 1) % self.sample_count;

                // Our random function only generates from 0 - 1.  We can make
                // it distribute equally by multiplying by 2 and subtracting by 1.
                let r = 2f32 * (self.white_noise[self.position]) - 1f32;

                let value: F = ((r * AMPLIFY_MULTIPLIER) * (self.velocity as f32 / 127f32)).as_();

                // Set our output buffer
                // This works both in a monophonic context and a polyphonic context.
                M::set(sample, value);
            }
        }
    }
}

impl<M, C> EventHandler<Timed<RawMidiEvent>, C> for Sound<M>
where
    M: OutputMode,
{
    fn handle_event(&mut self, timed: Timed<RawMidiEvent>, _context: &mut C) {
        trace!("handle_event(event: ...)"); // TODO: Should events implement Debug?

        // We currently ignore the `time_in_frames` field.
        // There are some vague plans to add middleware that makes it easier
        // to make sample-accurate plugins.
        // As a developer, we are simply waiting for that, so right
        // now it's not sample-accurate.
        let state_and_chanel = timed.event.data()[0];

        // We are digging into the details of midi-messages here.
        // There are some vague plans to make this easier in the future
        // as well. For now, let's do some bits masking:
        if state_and_chanel & 0xF0 == 0x90 {
            self.is_playing = true;
            self.velocity = timed.event.data()[2];
        }
        if state_and_chanel & 0xF0 == 0x80 {
            self.velocity = 0;
            self.is_playing = false;
        }
    }
}

impl<'a, M, C> EventHandler<Timed<SysExEvent<'a>>, C> for Sound<M>
where
    M: OutputMode,
{
    fn handle_event(&mut self, _event: Timed<SysExEvent<'a>>, context: &mut C) {
        // We don't do anything with SysEx events.
    }
}

// This enables using Sound in a polyphonic context.
impl<M> Voice for Sound<M>
where
    M: OutputMode,
{
    fn is_playing(&self) -> bool {
        self.is_playing
    }
}

// Initialize the logging.
pub fn initialize_logging() {
    let mut unrecognized_log_level = None;
    let log_level = match env::var("RSYNTH_LOG_LEVEL") {
        Err(_) => LevelFilter::Error,
        Ok(s) => match s.as_ref() {
            "off" => LevelFilter::Off,
            "error" => LevelFilter::Error,
            "warning" => LevelFilter::Warn,
            "info" => LevelFilter::Info,
            "debug" => LevelFilter::Debug,
            "trace" => LevelFilter::Trace,
            &_ => {
                unrecognized_log_level = Some(s.clone());
                LevelFilter::Error
            }
        },
    };
    let log_file = match env::var("RSYNTH_LOG_FILE") {
        Err(env::VarError::NotPresent) => {
            return;
        }
        Err(env::VarError::NotUnicode(os_string)) => {
            match File::create(os_string) {
                Ok(f) => f,
                Err(_) => {
                    // There is not much that we can do here.
                    // We even cannot log this :-(
                    // TODO: Use better error handling.
                    return;
                }
            }
        }
        Ok(s) => {
            match File::create(s) {
                Ok(f) => f,
                Err(_) => {
                    // There is not much that we can do here.
                    // We even cannot log this :-(
                    // TODO: Use better error handling.
                    return;
                }
            }
        }
    };
    WriteLogger::init(log_level, Config::default(), log_file);
    if let Some(unrecognized) = unrecognized_log_level {
        error!("`{}` is an unrecognized log level. Falling back to log level 'error'. Recognized log levels are: 'off', 'error', 'warning', 'info', 'debug' and 'trace'.", unrecognized);
    }
}
