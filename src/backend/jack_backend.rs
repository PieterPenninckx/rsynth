//! Wrapper for the [JACK] backend.
//! Support is only enabled if you compile with the "backend-jack" feature, see
//! [the cargo reference] for more information on setting cargo features.
//!
//! For an example, see `jack_synth.rs` in the `examples` folder.
//! `examples/test_synth.rs` contains the code that is shared for all backends and
//! `examples/jack_synth.rs` contains the jack-specific code.
//!
//! [JACK]: http://www.jackaudio.org/
//! [the cargo reference]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-features-section
use crate::backend::{HostInterface, Stop};
use crate::buffer::AudioBufferInOut;
use crate::event::{
    ContextualEventHandler, EventHandler, Indexed, RawMidiEvent, SysExEvent, Timed,
};
use crate::{
    AudioHandler, CommonAudioPortMeta, CommonMidiPortMeta, CommonPluginMeta,
    ContextualAudioRenderer,
};
use core::cmp;
use jack::{AudioIn, AudioOut, MidiIn, MidiOut, Port, ProcessScope, RawMidi};
use jack::{Client, ClientOptions, Control, ProcessHandler};
use std::io;
use std::slice;
use vecstorage::VecStorage;

pub struct JackHost<'c, 'mp, 'mw> {
    // Note: the `_client` field is currently not used, but is is rather likely that it
    // will be used in the future. Because it may introduce an extra complexity because of
    // the lifetime, we keep it, so that we can keep track of this complexity.
    _client: &'c Client,
    midi_out_ports: &'mp mut [jack::MidiWriter<'mw>],
    control: jack::Control,
}

impl<'c, 'mp, 'mw> HostInterface for JackHost<'c, 'mp, 'mw> {
    fn output_initialized(&self) -> bool {
        false
    }

    fn stop(&mut self) {
        self.control = jack::Control::Quit
    }
}

impl<'c, 'mp, 'mw> Stop for JackHost<'c, 'mp, 'mw> {}

impl<'c, 'mp, 'mw> EventHandler<Indexed<Timed<RawMidiEvent>>> for JackHost<'c, 'mp, 'mw> {
    fn handle_event(&mut self, event: Indexed<Timed<RawMidiEvent>>) {
        let Indexed { index, event } = event;
        if let Some(ref mut midi_out_port) = self.midi_out_ports.get_mut(index).as_mut() {
            let raw_midi = RawMidi {
                time: event.time_in_frames,
                bytes: event.event.data(),
            };
            midi_out_port.write(&raw_midi); // TODO: error handling.
        } else {
            error!(
                "midi port out of bounds: port index is {}, but only {} ports are available",
                index,
                self.midi_out_ports.len()
            );
        }
    }
}

impl<'c, 'mp, 'mw, 'e> EventHandler<Indexed<Timed<SysExEvent<'e>>>> for JackHost<'c, 'mp, 'mw> {
    fn handle_event(&mut self, event: Indexed<Timed<SysExEvent>>) {
        let Indexed { index, event } = event;
        if let Some(ref mut midi_out_port) = self.midi_out_ports.get_mut(index).as_mut() {
            let raw_midi = RawMidi {
                time: event.time_in_frames,
                bytes: event.event.data(),
            };
            midi_out_port.write(&raw_midi); // TODO: error handling.
        } else {
            error!(
                "midi port out of bounds: port index is {}, but only {} ports are available",
                index,
                self.midi_out_ports.len()
            );
        }
    }
}

fn audio_in_ports<P>(client: &Client, plugin: &P) -> Vec<Port<AudioIn>>
where
    P: CommonAudioPortMeta,
{
    let mut in_ports = Vec::with_capacity(plugin.max_number_of_audio_inputs());
    for index in 0..plugin.max_number_of_audio_inputs() {
        let name = plugin.audio_input_name(index);
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

fn audio_out_ports<P>(client: &Client, plugin: &P) -> Vec<Port<AudioOut>>
where
    P: CommonAudioPortMeta,
{
    let mut out_ports = Vec::with_capacity(plugin.max_number_of_audio_outputs());
    for index in 0..plugin.max_number_of_audio_outputs() {
        let name = plugin.audio_output_name(index);
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

fn midi_in_ports<P>(client: &Client, plugin: &P) -> Vec<Port<MidiIn>>
where
    P: CommonMidiPortMeta,
{
    let mut in_ports = Vec::with_capacity(plugin.max_number_of_midi_inputs());
    for index in 0..plugin.max_number_of_midi_inputs() {
        let name = plugin.midi_input_name(index);
        info!("Registering midi input port with name {}", name);
        let port = client.register_port(&name, MidiIn::default());
        match port {
            Ok(p) => {
                in_ports.push(p);
            }
            Err(e) => {
                // TODO: Maybe instead of skipping, it is better to provide a "dummy" midi input port?
                error!("Failed to open midi input port with index {} and name {}: {:?}. Skipping this port.", index, name, e);
            }
        }
    }
    in_ports
}

fn midi_out_ports<P>(client: &Client, plugin: &P) -> Vec<Port<MidiOut>>
where
    P: CommonMidiPortMeta,
{
    let mut out_ports = Vec::with_capacity(plugin.max_number_of_midi_outputs());
    for index in 0..plugin.max_number_of_midi_outputs() {
        let name = plugin.midi_output_name(index);
        info!("Registering midi output port with name {}", name);
        let port = client.register_port(&name, MidiOut::default());
        match port {
            Ok(p) => {
                out_ports.push(p);
            }
            Err(e) => {
                // TODO: Maybe instead of skipping, it is better to provide a "dummy" midi output port?
                error!("Failed to open midi output port with index {} and name {}: {:?}. Skipping this port.", index, name, e);
            }
        }
    }
    out_ports
}

// `MidiWriter` does not implement `Send`, but we do want `JackProcessHandler` to implement `Send`.
// `JackProcessHandler` contains only `VecStorage` of `MidiWriter`s, not a real `MidiWriter`.
// So we solve this by creating a data type that is guaranteed to have the same alignment and
// size as a `MidiWriter`.
struct MidiWriterWrapper {
    _inner: jack::MidiWriter<'static>,
}

unsafe impl Send for MidiWriterWrapper {}
unsafe impl Sync for MidiWriterWrapper {}

struct JackProcessHandler<P> {
    audio_in_ports: Vec<Port<AudioIn>>,
    audio_out_ports: Vec<Port<AudioOut>>,
    midi_in_ports: Vec<Port<MidiIn>>,
    midi_out_ports: Vec<Port<MidiOut>>,
    plugin: P,
    inputs: VecStorage<&'static [f32]>,
    outputs: VecStorage<&'static [f32]>,
    midi_writer: VecStorage<MidiWriterWrapper>,
}

impl<P> JackProcessHandler<P>
where
    P: CommonAudioPortMeta + CommonMidiPortMeta + CommonPluginMeta + Send,
    for<'c, 'mp, 'mw> P: ContextualAudioRenderer<f32, JackHost<'c, 'mp, 'mw>>
        + ContextualEventHandler<Indexed<Timed<RawMidiEvent>>, JackHost<'c, 'mp, 'mw>>,
    for<'c, 'mp, 'mw, 'a> P:
        ContextualEventHandler<Indexed<Timed<SysExEvent<'a>>>, JackHost<'c, 'mp, 'mw>>,
{
    fn new(client: &Client, plugin: P) -> Self {
        trace!("JackProcessHandler::new()");
        let audio_in_ports = audio_in_ports::<P>(&client, &plugin);
        let audio_out_ports = audio_out_ports::<P>(&client, &plugin);

        let midi_in_ports = midi_in_ports::<P>(&client, &plugin);
        let midi_out_ports = midi_out_ports::<P>(&client, &plugin);

        let inputs = VecStorage::with_capacity(plugin.max_number_of_audio_inputs());
        let outputs = VecStorage::with_capacity(plugin.max_number_of_audio_outputs());

        let midi_writer = VecStorage::with_capacity(plugin.max_number_of_midi_outputs());

        JackProcessHandler {
            audio_in_ports,
            audio_out_ports,
            midi_in_ports,
            midi_out_ports,
            plugin,
            inputs,
            outputs,
            midi_writer,
        }
    }

    fn handle_events<'c, 'mp, 'mw>(
        midi_in_ports: &[Port<MidiIn>],
        plugin: &mut P,
        process_scope: &ProcessScope,
        jack_host: &mut JackHost<'c, 'mp, 'mw>,
    ) {
        // No tracing here, because this is called in the `process` function,
        // and we do not want to trace that.
        for (index, midi_in_port) in midi_in_ports.iter().enumerate() {
            trace!("handle_events for input port {}", index);
            for input_event in midi_in_port.iter(process_scope) {
                trace!("handle_events found event: {:?}", &input_event.bytes);
                if input_event.bytes.len() <= 3 {
                    if let Some(raw_event) = RawMidiEvent::try_new(&input_event.bytes) {
                        let event = Indexed {
                            index,
                            event: Timed {
                                time_in_frames: input_event.time,
                                event: raw_event,
                            },
                        };
                        plugin.handle_event(event, jack_host);
                    } else {
                        warn!(
                            "Strange event of length {}; ignoring this event.",
                            input_event.bytes.len()
                        );
                    }
                } else {
                    let event = Indexed {
                        index,
                        event: Timed {
                            time_in_frames: input_event.time,
                            event: SysExEvent::new(input_event.bytes),
                        },
                    };
                    plugin.handle_event(event, jack_host);
                }
            }
        }
    }
}

impl<P> ProcessHandler for JackProcessHandler<P>
where
    P: CommonAudioPortMeta + CommonMidiPortMeta + CommonPluginMeta + Send,
    for<'c, 'mp, 'mw> P: ContextualAudioRenderer<f32, JackHost<'c, 'mp, 'mw>>
        + ContextualEventHandler<Indexed<Timed<RawMidiEvent>>, JackHost<'c, 'mp, 'mw>>,
    for<'c, 'mp, 'mw, 'a> P:
        ContextualEventHandler<Indexed<Timed<SysExEvent<'a>>>, JackHost<'c, 'mp, 'mw>>,
{
    fn process(&mut self, client: &Client, process_scope: &ProcessScope) -> Control {
        let mut midi_writer_guard = self.midi_writer.vec_guard();
        for midi_output in self.midi_out_ports.iter_mut() {
            midi_writer_guard.push(midi_output.writer(process_scope));
        }
        let mut jack_host: JackHost = JackHost {
            _client: client,
            midi_out_ports: midi_writer_guard.as_mut_slice(),
            control: jack::Control::Continue,
        };
        Self::handle_events(
            &self.midi_in_ports,
            &mut self.plugin,
            process_scope,
            &mut jack_host,
        );

        let mut inputs = self.inputs.vec_guard();
        for i in 0..cmp::min(self.audio_in_ports.len(), inputs.capacity()) {
            inputs.push(self.audio_in_ports[i].as_slice(process_scope));
        }

        let mut outputs = self.outputs.vec_guard();
        let number_of_frames = process_scope.n_frames();
        for i in 0..cmp::min(self.audio_out_ports.len(), outputs.capacity()) {
            // We use some unsafe here because otherwise, the compiler believes we are borrowing
            // `self.audio_out_ports` multiple times.
            let buffer = unsafe {
                slice::from_raw_parts_mut(
                    self.audio_out_ports[i].buffer(number_of_frames) as *mut f32,
                    number_of_frames as usize,
                )
            };
            outputs.push(buffer);
        }

        let mut buffer = AudioBufferInOut::new(
            inputs.as_slice(),
            outputs.as_mut_slice(),
            client.buffer_size() as usize,
        );
        self.plugin.render_buffer(&mut buffer, &mut jack_host);
        jack_host.control
    }
}

/// Run the plugin until the user presses a key on the computer keyboard.
pub fn run<P>(mut plugin: P) -> Result<P, jack::Error>
where
    P: CommonAudioPortMeta
        + AudioHandler
        + CommonMidiPortMeta
        + CommonPluginMeta
        + Send
        + Sync
        + 'static,
    for<'c, 'mp, 'mw> P: ContextualAudioRenderer<f32, JackHost<'c, 'mp, 'mw>>
        + ContextualEventHandler<Indexed<Timed<RawMidiEvent>>, JackHost<'c, 'mp, 'mw>>,
    for<'c, 'mp, 'mw, 'a> P:
        ContextualEventHandler<Indexed<Timed<SysExEvent<'a>>>, JackHost<'c, 'mp, 'mw>>,
{
    let (client, _status) = Client::new(plugin.name(), ClientOptions::NO_START_SERVER)?;

    let sample_rate = client.sample_rate();
    plugin.set_sample_rate(sample_rate as f64);

    let jack_process_handler = JackProcessHandler::new(&client, plugin);
    let active_client = client.activate_async((), jack_process_handler)?;

    println!("Press any key to quit");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();

    info!("Deactivating client...");

    let (_, _, plugin) = active_client.deactivate()?;
    return Ok(plugin.plugin);
}
