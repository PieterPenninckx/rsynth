//! Wrapper for the [JACK] backend (behind the `backend-jack` feature).
//!
//! Support is only enabled if you compile with the "backend-jack" feature, see
//! [the cargo reference] for more information on setting cargo features.
//!
//! For an example, see `jack_synth.rs` in the `examples` folder.
//! `examples/example_synth` contains the code that is shared for all backends and
//! `examples/jack_synth.rs` contains the jack-specific code.
//!
//! # Usage
//! See the documentation of the [`run`] function.
//!
//! [JACK]: http://www.jackaudio.org/
//! [the cargo reference]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-features-section
//! [`run`]: ./fn.run.html
use crate::backend::{HostInterface, Stop};
use crate::buffer::{AudioBufferInOut, DelegateHandling};
use crate::event::{
    CoIterator, ContextualEventHandler, EventHandler, Indexed, RawMidiEvent, SysExEvent, Timed,
};
use crate::{
    AudioHandler, CommonAudioPortMeta, CommonMidiPortMeta, CommonPluginMeta,
    ContextualAudioRenderer,
};
use std::io;
use vecstorage::VecStorage;

/// Re-exports of the [`jack`](https://crates.io/crates/jack) crate.
/// Use this so that your code doesn't break when `rsynth` upgrades its dependency on `jack`.
pub mod jack {
    pub use jack::*;
}

use self::jack::{AudioIn, AudioOut, MidiIn, MidiOut, Port, ProcessScope, RawMidi};
use self::jack::{Client, ClientOptions, Control, ProcessHandler};
use crate::backend::jack_backend::jack::{Error, MidiWriter};
use std::convert::TryFrom;

/// _Note_: you have to be very specific with references here,
/// e.g.
/// ```
/// use rsynth::backend::jack_backend::jack::RawMidi;
/// use rsynth::event::{RawMidiEvent, Timed};
///
/// fn jack_function<'a>(raw: &RawMidi<'a>) {
/// }
///
/// fn my_function() {
///     let raw_midi: Timed<RawMidiEvent> = todo!();
///     // Note the explicit references on the next line.
///     jack_function(&((&item).into()));
/// }
/// ```
impl<'a> Into<RawMidi<'a>> for &'a Timed<RawMidiEvent> {
    fn into(self) -> RawMidi<'a> {
        RawMidi {
            time: self.time_in_frames as u32,
            bytes: self.event.bytes(),
        }
    }
}

impl<'c> CoIterator<Timed<RawMidiEvent>> for MidiWriter<'c> {
    fn co_next(&mut self, item: Timed<RawMidiEvent>) {
        // Not yet found a way to handle errors :-(
        let _ = self.write(&((&item).into()));
    }
}

/// Used to communicate with `Jack`.
///
/// You don't need to instantiate this yourself: it is passed as the `context`
/// parameter to the [`render_audio`] method when using the [`run`] function.
///
/// [`render_audio`]: ../../trait.ContextualAudioRenderer.html#tymethod.render_buffer
/// [`run`]: ./fn.run.html
pub struct JackHost<'c, 'mp, 'mw> {
    client: &'c Client,
    midi_out_ports: &'mp mut [jack::MidiWriter<'mw>],
    control: jack::Control,
}

impl<'c, 'mp, 'mw> JackHost<'c, 'mp, 'mw> {
    /// Get access to the underlying [`Client`] so that you can use Jack-specific features.
    ///
    /// ['Client`]: ./jack/struct.Client.html
    pub fn client(&self) -> &'c Client {
        self.client
    }
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
                bytes: event.event.bytes(),
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
        let mut name = String::new();
        if let Err(e) = plugin.input_name(&mut name, index) {
            error!(
                "Failed to get the name of audio input port with index {}: {}.",
                index, e
            );
            // TODO: Maybe instead of skipping, it is better to provide a "dummy" audio input port?
            continue;
        }
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
        let mut name = String::new();
        if let Err(e) = plugin.output_name(&mut name, index) {
            error!(
                "Failed to get the name of audio output port with index {}: {}.",
                index, e
            );
            // TODO: Maybe instead of skipping, it is better to provide a "dummy" audio output port?
            continue;
        }
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
        let mut name = String::new();
        if let Err(e) = plugin.input_name(&mut name, index) {
            error!(
                "Failed to get the name of midi input port with index {}: {}.",
                index, e
            );
            // TODO: Maybe instead of skipping, it is better to provide a "dummy" midi input port?
            continue;
        }
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
        let mut name = String::new();
        if let Err(e) = plugin.output_name(&mut name, index) {
            error!(
                "Failed to get the name of midi output port with index {}: {}.",
                index, e
            );
            // TODO: Maybe instead of skipping, it is better to provide a "dummy" midi output port?
            continue;
        }
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
    midi_writer: VecStorage<MidiWriterWrapper>, // We cannot use rsor for this one.
}

impl<P> JackProcessHandler<P>
where
    P: CommonAudioPortMeta + CommonMidiPortMeta + Send,
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
            client,
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
        for port in self.audio_in_ports.iter().take(inputs.capacity()) {
            inputs.push(port.as_slice(process_scope));
        }

        let mut outputs = self.outputs.vec_guard();
        for port in self.audio_out_ports.iter_mut().take(outputs.capacity()) {
            outputs.push(port.as_mut_slice(process_scope));
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

/// Run the plugin until the user presses a key on the computer keyboard or the plugin
/// requests the `JackHost` to stop.
pub fn run<P>(mut plugin: P) -> Result<P, jack::Error>
where
    P: CommonPluginMeta
        + AudioHandler
        + CommonAudioPortMeta
        + CommonMidiPortMeta
        + Send
        + Sync
        + 'static,
    for<'c, 'mp, 'mw> P: ContextualAudioRenderer<f32, JackHost<'c, 'mp, 'mw>>,
    for<'c, 'mp, 'mw> P:
        ContextualEventHandler<Indexed<Timed<RawMidiEvent>>, JackHost<'c, 'mp, 'mw>>,
    for<'c, 'mp, 'mw, 'a> P:
        ContextualEventHandler<Indexed<Timed<SysExEvent<'a>>>, JackHost<'c, 'mp, 'mw>>,
{
    let mut client_name = String::new();
    plugin.plugin_name(&mut client_name);
    let (client, _status) = Client::new(&client_name, ClientOptions::NO_START_SERVER)?;

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

macro_rules! derive_stuff {
    (
        $(#[$global_meta:meta])*
        struct $buffer_name:ident$(<$lt:lifetime>)?
        {
            $global_head:tt
            $($global_tail:tt)*
        }
        $(
            $(#[$local_meta:meta])*
            $local_macro:ident!{
                $($local_token:tt)*
            }
        )*
    ) => {
        $(#[$global_meta])*
        struct $buffer_name$(<$lt>)?
        {
            $global_head
            $($global_tail)*
        }
        derive_stuff!{
            @inner
            $buffer_name
            @($($global_tail)*)
            @(
                $(
                    $(#[$local_meta])*
                    $local_macro!{
                        @($global_head)
                        @($($local_token)*)
                    }
                )*
            )
        }
    };
    (
        @inner
        $buffer_name:ident
        @()
        @(
            $(
                $(#[$local_meta:meta])*
                $local_macro:ident!{
                    @($($global_processed:tt)*)
                    @($($local_token:tt)*)
                }
            )*
        )
    ) => {
        $(
            $(#[$local_meta])*
            $local_macro!{
                $buffer_name
                $(#[$local_meta])*
                @($($global_processed)*)
                @($($local_token)*)
            }
        )*
    };
    (
        @inner
        $buffer_name:ident
        @($global_head:tt $($global_tail:tt)*)
        @(
            $(
                $(#[$local_meta:meta])*
                $local_macro:ident!{
                    @($($global_processed:tt)*)
                    @($($local_token:tt)*)
                }
            )*
        )
    ) => {
        derive_stuff!{
            @inner
            $buffer_name
            @($($global_tail)*)
            @(
                $(
                    $(#[$local_meta])*
                    $local_macro!{
                        @($($global_processed)* $global_head)
                        @($($local_token)*)
                    }
                )*
            )
        }
    };
}

// TODO's:
// * Correctly take into account lifetime parameters of buffer
// * Add full paths to items defined by rsynth.
macro_rules! derive_jack_stuff {
    (
        $buffer_name:ident
        $(#[$local_meta:meta])*
        @($($global_tail:tt)*)
        @(struct $builder_name:ident { generate_fields!()})
    ) => {
        derive_jack_stuff!{
            @inner
            $buffer_name
            $builder_name
            $(#[$local_meta:meta])*
            @($($global_tail)*)
            @(process_scope, self)
            @()
            @()
            @()
        }
    };
    (
        @inner
        $buffer_name:ident
        $builder_name:ident
        $(#[$local_meta:meta])*
        @($(,)?)
        @($process_scope:ident, $self_:tt)
        @($($struct_constructor:tt)*)
        @($(($try_from_field_name:ident, $value:expr))*)
        @($($delegate_things: tt)*)
    ) => {
        $(#[$local_meta:meta])*
        struct $builder_name {
            $($struct_constructor)*
        }

        impl<'c> ::std::convert::TryFrom<&'c $crate::backend::jack_backend::jack::Client> for $builder_name {
            type Error = $crate::backend::jack_backend::jack::Error;

            fn try_from(
                client: &'c $crate::backend::jack_backend::jack::Client
            ) -> ::core::result::Result<Self, Self::Error> {
                Ok(Self {
                    $(
                        $try_from_field_name: client.register_port(stringify!($try_from_field_name), $value)?,
                    )*
                })
            }
        }

        impl<'a, P> DelegateHandling<P, (&'a $crate::backend::jack_backend::jack::Client, &'a $crate::backend::jack_backend::jack::ProcessScope)> for $builder_name
        where
            for<'b, 'c, 'mp, 'mw> P:
                NewContextualAudioRenderer<$buffer_name<'b>, JackHost<'c, 'mp, 'mw>>,
        {
            type Output = $crate::backend::jack_backend::jack::Control;

            fn delegate_handling(
                &mut $self_,
                plugin: &mut P,
                (client, $process_scope): (&'a $crate::backend::jack_backend::jack::Client, &'a $crate::backend::jack_backend::jack::ProcessScope),
            ) -> Self::Output {
                use ::std::convert::TryFrom;
                let mut jack_host: JackHost = JackHost {
                    client,
                    midi_out_ports: &mut [],
                    control: jack::Control::Continue,
                };

                let buffer = $buffer_name {
                    $(
                        $delegate_things
                    )*
                };
                plugin.render_buffer(buffer, &mut jack_host);
                jack_host.control
            }
        }
    };
    (
        @inner
        $buffer_name:ident
        $builder_name:ident
        $(#[$local_meta:meta])*
        @($(,)? $field_name:ident : &$lt:lifetime[f32] $($global_tail:tt)*)
        @($process_scope:ident, $self_:tt)
        @($($struct_constructor:tt)*)
        @($($try_from:tt)*)
        @($($delegate_things: tt)*)
    ) => {
        derive_jack_stuff!{
            @inner
            $buffer_name
            $builder_name
            $(#[$local_meta:meta])*
            @($($global_tail)*)
            @($process_scope, $self_)
            @($($struct_constructor)* $field_name : $crate::backend::jack_backend::jack::Port<AudioIn>,)
            @($($try_from)* ($field_name, $crate::backend::jack_backend::jack::AudioIn::default()))
            @($($delegate_things)* $field_name: $self_.$field_name.as_slice($process_scope),)
        }
    };
    (
        @inner
        $buffer_name:ident
        $builder_name:ident
        $(#[$local_meta:meta])*
        @($(,)? $field_name:ident : &$lt:lifetime mut[f32] $($global_tail:tt)*)
        @($process_scope:ident, $self_:tt)
        @($($struct_constructor:tt)*)
        @($($try_from:tt)*)
        @($($delegate_things: tt)*)
    ) => {
        derive_jack_stuff!{
            @inner
            $buffer_name
            $builder_name
            $(#[$local_meta:meta])*
            @($($global_tail)*)
            @($process_scope, $self_)
            @($($struct_constructor)* $field_name : $crate::backend::jack_backend::jack::Port<AudioOut>,)
            @($($try_from)* ($field_name, $crate::backend::jack_backend::jack::AudioOut::default()))
            @($($delegate_things)* $field_name: $self_.$field_name.as_mut_slice($process_scope),)
        }
    };
    (
        @inner
        $buffer_name:ident
        $builder_name:ident
        $(#[$local_meta:meta])*
        @($(,)? $field_name:ident : &$lt:lifetime mut dyn Iterator<Item = Timed<RawMidiEvent>> $($global_tail:tt)*)
        @($process_scope:ident, $self_:tt)
        @($($struct_constructor:tt)*)
        @($($try_from:tt)*)
        @($($delegate_things: tt)*)
    ) => {
        derive_jack_stuff!{
            @inner
            $buffer_name
            $builder_name
            $(#[$local_meta:meta])*
            @($($global_tail)*)
            @($process_scope, $self_)
            @($($struct_constructor)* $field_name : $crate::backend::jack_backend::jack::Port<MidiIn>,)
            @($($try_from)* ($field_name, $crate::backend::jack_backend::jack::MidiIn::default()))
            @($($delegate_things)*
                $field_name: &mut $self_.$field_name
                    .iter($process_scope)
                    .filter_map(|e| $crate::event::Timed::<$crate::event::RawMidiEvent>::try_from(e).ok()),
            )
        }
    };
}

derive_stuff! {
struct StereoInputOutput<'a> {
    in_left: &'a [f32],
    in_right: &'a [f32],
    out_left: &'a mut [f32],
    out_right: &'a mut [f32],
    midi_in: &'a mut dyn Iterator<Item = Timed<RawMidiEvent>>,
}

    derive_jack_stuff!{
        struct MyBuilder {generate_fields!() }
    }
}

struct StereoBuilder {
    in_left: Port<AudioIn>,
    in_right: Port<AudioIn>,
    out_left: Port<AudioOut>,
    out_right: Port<AudioOut>,
    midi_in: Port<MidiIn>,
}

impl<'c> TryFrom<&'c Client> for StereoBuilder {
    type Error = Error;

    fn try_from(client: &'c Client) -> Result<Self, Self::Error> {
        let in_left = client.register_port("left in", AudioIn::default())?;
        let in_right = client.register_port("right in", AudioIn::default())?;
        let out_left = client.register_port("left out", AudioOut::default())?;
        let out_right = client.register_port("right out", AudioOut::default())?;
        let midi_in = client.register_port("midi in", MidiIn::default())?;
        Ok(Self {
            in_left,
            in_right,
            out_left,
            out_right,
            midi_in,
        })
    }
}

pub trait NewContextualAudioRenderer<B, Context> {
    fn render_buffer(&mut self, buffer: B, context: &mut Context);
}

impl<'a, P> DelegateHandling<P, (&'a Client, &'a ProcessScope)> for StereoBuilder
where
    for<'b, 'c, 'mp, 'mw> P:
        NewContextualAudioRenderer<StereoInputOutput<'b>, JackHost<'c, 'mp, 'mw>>,
{
    type Output = Control;

    fn delegate_handling(
        &mut self,
        plugin: &mut P,
        (client, process_scope): (&'a Client, &'a ProcessScope),
    ) -> Self::Output {
        let mut jack_host: JackHost = JackHost {
            client,
            midi_out_ports: &mut [],
            control: jack::Control::Continue,
        };

        let in_left = self.in_left.as_slice(process_scope);
        let in_right = self.in_right.as_slice(process_scope);
        let out_left = self.out_left.as_mut_slice(process_scope);
        let out_right = self.out_right.as_mut_slice(process_scope);
        let mut midi_in = self
            .midi_in
            .iter(process_scope)
            .filter_map(|e| Timed::<RawMidiEvent>::try_from(e).ok());

        let buffer = StereoInputOutput {
            in_left,
            in_right,
            out_left,
            out_right,
            midi_in: &mut midi_in,
        };
        plugin.render_buffer(buffer, &mut jack_host);
        jack_host.control
    }
}

struct DemoJackHandler<B, P> {
    builder: B,
    plugin: P,
}

impl<B, P> ProcessHandler for DemoJackHandler<B, P>
where
    P: AudioHandler,
    for<'a> B: DelegateHandling<P, (&'a Client, &'a ProcessScope), Output = Control>,
    B: Send,
    P: Send,
{
    fn process(&mut self, client: &Client, process_scope: &ProcessScope) -> Control {
        self.builder
            .delegate_handling(&mut self.plugin, (client, process_scope))
    }
}

pub fn run2<P, B>(mut plugin: P) -> Result<P, jack::Error>
where
    P: CommonPluginMeta + AudioHandler + Send + Sync + 'static,
    for<'b, 'c, 'mp, 'mw> P:
        NewContextualAudioRenderer<StereoInputOutput<'b>, JackHost<'c, 'mp, 'mw>>,
    for<'a> B: DelegateHandling<P, (&'a Client, &'a ProcessScope), Output = Control>,
    for<'a> B: TryFrom<&'a Client, Error = jack::Error>,
    B: Send + 'static,
{
    let mut client_name = String::new();
    plugin.plugin_name(&mut client_name);
    let (client, _status) = Client::new(&client_name, ClientOptions::NO_START_SERVER)?;

    let sample_rate = client.sample_rate();
    plugin.set_sample_rate(sample_rate as f64);

    let stereo_builder = B::try_from(&client)?;

    let demo_handler = DemoJackHandler {
        plugin: plugin,
        builder: stereo_builder,
    };

    let active_client = client.activate_async((), demo_handler)?;

    println!("Press any key to quit");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();

    info!("Deactivating client...");

    let (_, _, plugin) = active_client.deactivate()?;
    return Ok(plugin.plugin);
}
