//! Wrapper for the [JACK] backend.
//! Support is only enabled if you compile with the "jack-backend" feature, see
//! [the cargo reference] for more information on setting cargo features.
//!
//! For an example, see `jack_synth.rs` in the `examples` folder.
//! `examples/test_synth.rs` contains the code that is shared for all backends and
//! `examples/jack_synth.rs` contains the jack-specific code.
//!
//! [JACK]: http://www.jackaudio.org/
//! [the cargo reference]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-features-section
use crate::backend::HostInterface;
use crate::dev_utilities::vecstorage::{VecStorage, VecStorageMut};
use crate::{
    event::{EventHandler, RawMidiEvent, Timed},
    Plugin,
};
use core::cmp;
use jack::{AudioIn, AudioOut, MidiIn, Port, ProcessScope};
use jack::{Client, ClientOptions, Control, ProcessHandler};
use std::io;
use std::slice;

impl<'c> HostInterface for &'c Client {}

fn audio_in_ports<P>(client: &Client) -> Vec<Port<AudioIn>>
where
    for<'c> P: Plugin<&'c Client>,
{
    let mut in_ports = Vec::with_capacity(P::MAX_NUMBER_OF_AUDIO_INPUTS);
    for index in 0..P::MAX_NUMBER_OF_AUDIO_INPUTS {
        let name = P::audio_input_name(index);
        info!("Registering audio input port with name {}", name);
        let port = client.register_port(&name, AudioIn::default());
        match port {
            Ok(p) => {
                in_ports.push(p);
            }
            Err(e) => {
                // TODO: Maybe instead of skipping, it is better to provide a "dummy" audio input
                // TODO: port that always contains silence?
                error!("Failed to open audio input port with index {} and name {}: {:?}. Skipping this port.", index, name, e);
            }
        }
    }
    in_ports
}

fn audio_out_ports<P>(client: &Client) -> Vec<Port<AudioOut>>
where
    for<'c> P: Plugin<&'c Client>,
{
    let mut out_ports = Vec::with_capacity(P::MAX_NUMBER_OF_AUDIO_OUTPUTS);
    for index in 0..P::MAX_NUMBER_OF_AUDIO_OUTPUTS {
        let name = P::audio_output_name(index);
        info!("Registering audio output port with name {}", name);
        let port = client.register_port(&name, AudioOut::default());
        match port {
            Ok(p) => {
                out_ports.push(p);
            }
            Err(e) => {
                // TODO: Maybe instead of skipping, it is better to provide a "dummy" audio output
                // TODO: port that is in fact unused?
                error!("Failed to open audio output port with index {} and name {}: {:?}. Skipping this port.", index, name, e);
            }
        }
    }
    out_ports
}

struct JackProcessHandler<P> {
    audio_in_ports: Vec<Port<AudioIn>>,
    audio_out_ports: Vec<Port<AudioOut>>,
    midi_in_port: Option<Port<MidiIn>>,
    plugin: P,
    inputs: VecStorage<[f32]>,
    outputs: VecStorageMut<[f32]>,
}

impl<P> JackProcessHandler<P>
where
    for<'c> P: Plugin<&'c Client> + EventHandler<Timed<RawMidiEvent>, &'c Client>,
{
    fn new(client: &Client, plugin: P) -> Self {
        trace!("JackProcessHandler::new()");
        let midi_in_port = match client.register_port("midi_in", MidiIn::default()) {
            Ok(mip) => Some(mip),
            Err(e) => {
                error!(
                    "Failed to open midi in port: {:?}. Continuing without midi input.",
                    e
                );
                None
            }
        };
        let audio_in_ports = audio_in_ports::<P>(&client);
        let audio_out_ports = audio_out_ports::<P>(&client);

        let inputs = VecStorage::with_capacity(P::MAX_NUMBER_OF_AUDIO_INPUTS);

        let outputs = VecStorageMut::with_capacity(P::MAX_NUMBER_OF_AUDIO_OUTPUTS);

        JackProcessHandler {
            audio_in_ports,
            audio_out_ports,
            midi_in_port,
            plugin,
            inputs,
            outputs,
        }
    }

    fn handle_events(&mut self, process_scope: &ProcessScope, client: &Client) {
        // No tracing here, because this is called in the `process` function,
        // and we do not want to trace that.
        if let Some(ref mut midi_in_port) = self.midi_in_port {
            for input_event in midi_in_port.iter(process_scope) {
                trace!("handle_events found event: {:?}", &input_event.bytes);
                if input_event.bytes.len() <= 3 {
                    let mut data = [0, 0, 0];
                    for i in 0..input_event.bytes.len() {
                        data[i] = input_event.bytes[i];
                    }
                    let event = Timed {
                        time_in_frames: input_event.time,
                        event: RawMidiEvent::new(data),
                    };
                    self.plugin.handle_event(event, &mut &*client);
                } else {
                    // TODO: SysEx event
                    // self.plugin.handle_event(event, &mut &*client);
                }
            }
        }
    }
}

impl<P> ProcessHandler for JackProcessHandler<P>
where
    P: Send,
    for<'c> P: Plugin<&'c Client> + EventHandler<Timed<RawMidiEvent>, &'c Client>,
{
    fn process(&mut self, client: &Client, process_scope: &ProcessScope) -> Control {
        self.handle_events(process_scope, client);

        let mut inputs = self.inputs.vec_guard();
        for i in 0..cmp::min(self.audio_in_ports.len(), inputs.capacity()) {
            inputs.push(self.audio_in_ports[i].as_slice(process_scope));
        }

        let mut outputs = self.outputs.vec_guard();
        let number_of_frames = process_scope.n_frames();
        for i in 0..cmp::min(self.audio_out_ports.len(), outputs.capacity()) {
            // We need to use some unsafe here because otherwise, the compiler believes
            // we are borrowing `self.audio_out_ports` multiple times.
            let buffer = unsafe {
                slice::from_raw_parts_mut(
                    self.audio_out_ports[i].buffer(number_of_frames) as *mut f32,
                    number_of_frames as usize,
                )
            };
            outputs.push(buffer);
        }

        self.plugin
            .render_buffer(inputs.as_slice(), outputs.as_mut_slice(), &mut &*client);

        Control::Continue
    }
}

/// Run the plugin until the user presses a key on the computer keyboard.
pub fn run<P>(mut plugin: P)
where
    P: Send,
    for<'c> P: Plugin<&'c Client> + EventHandler<Timed<RawMidiEvent>, &'c Client>,
{
    let (client, _status) = Client::new(P::NAME, ClientOptions::NO_START_SERVER).unwrap();

    let sample_rate = client.sample_rate();
    plugin.set_sample_rate(sample_rate as f64);

    //       For now, we keep the midi input ports (and name) hard-coded, but maybe we should
    //       probably define something like the following:
    //       ```
    //           pub trait JackPlugin : Plugin {
    //               const NUMBER_OF_MIDI_INPUTS: usize = 1; // Do we support defaults here?
    //               const NUMBER_OF_MIDI_OUTPUTS: usize = 0;
    //               fn midi_input_name(index: usize) -> String {
    //                   "midi_in".to_string()
    //               }
    //               fn midi_output_name(index: usize) -> String {
    //                   "midi_out".to_string()
    //               }
    //               fn handle_midi_in(&mut self, &SomeDataTypeAboutMidiInputPorts);
    //               fn handle_midi_out(&mut self, &mut SomeDataTypeAboutMidiOutputPorts);
    //           }
    //       And then the order to call the functions would be:
    //       1. handle_events   (is input for the plugin)
    //       2. handle_midi_in  (is input for the plugin)
    //       3. process_buffer  (is both input and output for the plugin,
    //                           must be after all other input and before all other output)
    //       4. handle_midi_out (is output for the plugin)
    //       ```
    let jack_process_handler = JackProcessHandler::new(&client, plugin);
    let active_client = match client.activate_async((), jack_process_handler) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to activate client: {:?}", e);
            return;
        }
    };

    println!("Press any key to quit");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();

    info!("Deactivating client...");
    match active_client.deactivate() {
        Ok(_) => {
            info!("Client deactivated.");
        }
        Err(e) => {
            error!("Failed to deactivate client: {:?}", e);
        }
    }
}

// Not yet needed because we do not yet have Jack-specific types.
/*
#[cfg(feature = "stable")]
impl_specialization!(
    trait NotInCrateRsynthFeatureJack;
    macro macro_for_rsynth_feature_jack;
);
*/
