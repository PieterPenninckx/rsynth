use std::slice;
use jack::{Port, AudioIn, AudioOut, ProcessScope, MidiIn};
use super::{Plugin, Event, RawMidiEvent};
use jack::{Client, ClientOptions, Control, ProcessHandler};
use core::cmp;
use std::io;
use backend::Hibernation;


fn audio_in_ports<P, E>(client: &Client) -> Vec<Port<AudioIn>>
where P: Plugin<E>
{
    let mut in_ports = Vec::with_capacity(P::MAX_NUMBER_OF_AUDIO_INPUTS);
    for index in 0 .. P::MAX_NUMBER_OF_AUDIO_INPUTS {
        let name = P::audio_input_name(index);
        let port = client.register_port(&name, AudioIn::default());
        match port {
            Ok(p) => {
                in_ports.push(p);
            },
            Err(e) => {
                error!("Failed to open audio input port with index {} and name {}: {:?}", index, name, e);
            }
        }
    }
    in_ports
}

fn audio_out_ports<P, E>(client: &Client) -> Vec<Port<AudioOut>>
    where P: Plugin<E>
{
    let mut out_ports = Vec::with_capacity(P::MAX_NUMBER_OF_AUDIO_OUTPUTS);
    for index in 0 .. P::MAX_NUMBER_OF_AUDIO_OUTPUTS {
        let name = P::audio_output_name(index);
        let port = client.register_port(&name, AudioOut::default());
        match port {
            Ok(p) => {
                out_ports.push(p);
            },
            Err(e) => {
                error!("Failed to open audio output port with index {} and name {}: {:?}", index, name, e);
            }
        }
    }
    out_ports
}

struct JackProcessHandler<P>
{
    audio_in_ports: Vec<Port<AudioIn>>,
    audio_out_ports: Vec<Port<AudioOut>>,
    midi_in_port: Option<Port<MidiIn>>,
    plugin: P,
    inputs: Hibernation,
    outputs: Hibernation
}

impl<P> JackProcessHandler<P>
where
    P: Send,
    for<'a> P: Plugin<Event<RawMidiEvent<'a>, ()>>
{
    fn new(client: &Client, plugin: P) -> Self {
        let midi_in_port = match client.register_port("midi_in", MidiIn::default()) {
            Ok(mip) => Some(mip),
            Err(e) => {
                error!("Failed to open mini in port: {:?}", e);
                None
            }
        };
        let audio_in_ports = audio_in_ports::<P, _>(&client);
        let audio_out_ports = audio_out_ports::<P, _>(&client);

        let inputs = Hibernation::new::<&[f32]>(P::MAX_NUMBER_OF_AUDIO_INPUTS);

        let outputs = Hibernation::new::<&mut[f32]>(P::MAX_NUMBER_OF_AUDIO_OUTPUTS);

        JackProcessHandler {
            audio_in_ports,
            audio_out_ports,
            midi_in_port,
            plugin,
            inputs,
            outputs
        }
    }

    fn handle_events(&mut self, process_scope: &ProcessScope) {
        if let Some(ref mut midi_in_port) = self.midi_in_port {
            for input_event in midi_in_port.iter(process_scope) {
                let raw_midi_event = RawMidiEvent {
                    data: input_event.bytes
                };
                let event = Event::Timed {
                    event: raw_midi_event,
                    samples: input_event.time
                };
                self.plugin.handle_event(&event);
            }
        }
    }
}

impl<P> ProcessHandler for JackProcessHandler<P>
where
    P: Send,
    for<'a> P: Plugin<Event<RawMidiEvent<'a>, ()>>
{
    fn process(&mut self, _client: &Client, process_scope: &ProcessScope) -> Control {
        self.handle_events(process_scope);
        // We avoid memory allocation in this piece of the code.
        // The slices themselves are allocated by Jack,
        // we only need a vector to store them in.
        // We allocate this vector upon creation, "wake it up" for each call to `process`
        // and let it "hibernate" between two calls to `process`.
        let mut inputs : Vec<&[f32]> = unsafe { self.inputs.wake_up() };
        for i in 0 .. cmp::min(self.audio_in_ports.len(), inputs.capacity()) {
            inputs.push(self.audio_in_ports[i].as_slice(process_scope));
        }

        let mut outputs: Vec<&mut[f32]> = unsafe { self.outputs.wake_up()};
        let number_of_frames = process_scope.n_frames();
        for i in 0 .. cmp::min(self.audio_out_ports.len(), outputs.capacity()) {
            let buffer = unsafe {
                slice::from_raw_parts_mut(
                    self.audio_out_ports[i].buffer(number_of_frames) as *mut f32,
                    number_of_frames as usize,
                )
            };
            outputs.push(buffer);
        }

        self.plugin.render_buffer(&inputs, &mut outputs);

        // Make sure to clear all input- and output slices before "hibernation".
        self.inputs.hibernate(inputs);
        self.outputs.hibernate(outputs);

        Control::Continue
    }
}

impl<P> Drop for JackProcessHandler<P>
{
    fn drop(&mut self) {
        unsafe {
            self.inputs.drop::<&[f32]>();
            self.outputs.drop::<&mut [f32]>();
        }
    }
}


// Run the plugin indefinitely. There is currently no way to stop it.
pub fn run<P>(plugin: P)
where
    P: Send,
    for<'a> P: Plugin<Event<RawMidiEvent<'a>, ()>>
{
    let (client, _status) =
        Client::new(P::NAME, ClientOptions::NO_START_SERVER).unwrap();
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
    match active_client.deactivate() {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to deactivate client: {:?}", e);
        }
    }
}
