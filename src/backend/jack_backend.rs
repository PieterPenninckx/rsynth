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
use crate::buffer::DelegateHandling;
use crate::event::{CoIterator, EventHandler, Indexed, RawMidiEvent, SysExEvent, Timed};
use crate::{AudioHandler, ContextualAudioRenderer};
use std::io;

/// Re-exports of the [`jack`](https://crates.io/crates/jack) crate.
/// Use this so that your code doesn't break when `rsynth` upgrades its dependency on `jack`.
pub mod jack {
    pub use jack::*;
}

use self::jack::{AudioIn, AudioOut, MidiIn, MidiOut, Port, ProcessScope, RawMidi};
use self::jack::{Client, ClientOptions, Control, ProcessHandler};
use crate::backend::jack_backend::jack::{Error, MidiIter, MidiWriter, PortSpec};
use std::convert::TryFrom;
use std::ops::{Deref, DerefMut};

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

impl<'c> CoIterator for MidiWriter<'c> {
    type Item = Timed<RawMidiEvent>;
    fn co_next(&mut self, item: Self::Item) {
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
// TODO: stop making fields public.
pub struct JackHost<'c, 'mp, 'mw> {
    pub client: &'c Client,
    pub midi_out_ports: &'mp mut [jack::MidiWriter<'mw>],
    pub control: jack::Control,
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

// TODO's:
// * Make the `derive_ports` macro also pass the token-tree with any lifetime replaced by `'static`
//   (preferably: first the own things, then the jack-specific things)
// * generate code as follows:
//   ```
//   struct MyBuilder {
//       my_field: <MyStaticType as JackPort>::Port
//   }
//   ```
//   and then for the delegation: `my_field.build(process_scope).my_into()` (see the `MyInto`
//   trait at the bottom, but probably use a better name).
//   Note: the `build` method should be a method on a type, not a method defined by a trait,
//   since you cannot (yet) do
//   ```
//   pub trait MyBuilder<'a> {
//       type Output;
//       fn build(self, process_scope: &'a ProcessScope) -> Self::Output;
//   }
//
//   impl<'a> MyBuilder<'a> for &'a mut Port<MidiOut> {
//       // Error on the next line: `impl Trait` in type aliases is unstable,
//       // See issue https://github.com/rust-lang/rust/issues/63063 for more information.
//       type Output = impl Iterator<Item = Timed<RawMidiEvent>> + 'a;
//
//       fn build(self, process_scope: &'a ProcessScope) -> Self::Output {
//           self.writer(process_scope)
//       }
//   }
//   ```
#[macro_export]
macro_rules! derive_jack_port_builder {
    (
        $buffer_name:ident
        $(#[$local_meta:meta])*
        @($($global_tail:tt)*)
        @(struct $builder_name:ident { $($whatever:tt)*})
    ) => {
        derive_jack_port_builder!{
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
        @($(($try_from_field_name:ident, $try_from_type:ty))*)
        @($(($field_name:ident, $temp:ident))*)
    ) => {
        $(#[$local_meta:meta])*
        pub struct $builder_name {
            $($struct_constructor)*
        }

        impl<'c> ::std::convert::TryFrom<&'c $crate::backend::jack_backend::jack::Client> for $builder_name {
            type Error = $crate::backend::jack_backend::jack::Error;

            fn try_from(
                client: &'c $crate::backend::jack_backend::jack::Client
            ) -> ::core::result::Result<Self, Self::Error> {
                Ok(Self {
                    $(
                        $try_from_field_name: <$try_from_type>::from_client(client, stringify!($try_from_field_name))?,
                    )*
                })
            }
        }

        impl<'a, P> $crate::buffer::DelegateHandling<P, (&'a $crate::backend::jack_backend::jack::Client, &'a $crate::backend::jack_backend::jack::ProcessScope)> for $builder_name
        where
            for<'b, 'c, 'mp, 'mw> P:
                $crate::ContextualAudioRenderer<$buffer_name<'b>, $crate::backend::jack_backend::JackHost<'c, 'mp, 'mw>>,
        {
            type Output = $crate::backend::jack_backend::jack::Control;

            fn delegate_handling(
                &mut self,
                plugin: &mut P,
                (client, process_scope): (&'a $crate::backend::jack_backend::jack::Client, &'a $crate::backend::jack_backend::jack::ProcessScope),
            ) -> Self::Output {
                use ::std::convert::TryFrom;
                let mut jack_host = $crate::backend::jack_backend::JackHost {
                    client,
                    midi_out_ports: &mut [],
                    control: jack::Control::Continue,
                };

                use $crate::backend::jack_backend::MyInto;
                $(
                    let mut $temp = self.$field_name.build(process_scope);
                )*
                let buffer = $buffer_name {
                    $(
                        $field_name: $temp.my_into(),
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
        derive_jack_port_builder!{
            @inner
            $buffer_name
            $builder_name
            $(#[$local_meta:meta])*
            @($($global_tail)*)
            @($process_scope, $self_)
            @($($struct_constructor)* $field_name : <&'static [f32] as $crate::backend::jack_backend::JackBuilder>::Port,)
            @($($try_from)* ($field_name, <&'static [f32] as $crate::backend::jack_backend::JackBuilder>::Port))
            @($($delegate_things)* ($field_name, temp))
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
        derive_jack_port_builder!{
            @inner
            $buffer_name
            $builder_name
            $(#[$local_meta:meta])*
            @($($global_tail)*)
            @($process_scope, $self_)
            @($($struct_constructor)* $field_name : <&'static mut [f32] as $crate::backend::jack_backend::JackBuilder>::Port,)
            @($($try_from)* ($field_name, <&'static mut [f32] as $crate::backend::jack_backend::JackBuilder>::Port))
            @($($delegate_things)* ($field_name, temp))
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
        derive_jack_port_builder!{
            @inner
            $buffer_name
            $builder_name
            $(#[$local_meta:meta])*
            @($($global_tail)*)
            @($process_scope, $self_)
            @($($struct_constructor)* $field_name : <&'static mut dyn Iterator<Item = Timed<RawMidiEvent>> as $crate::backend::jack_backend::JackBuilder>::Port,)
            @($($try_from)* ($field_name, <&'static mut dyn Iterator<Item = Timed<RawMidiEvent>> as $crate::backend::jack_backend::JackBuilder>::Port))
            @($($delegate_things)* ($field_name, temp))
        }
    };
    (
        @inner
        $buffer_name:ident
        $builder_name:ident
        $(#[$local_meta:meta])*
        @($(,)? $field_name:ident : &$lt:lifetime mut dyn CoIterator<Item = Timed<RawMidiEvent>> $($global_tail:tt)*)
        @($process_scope:ident, $self_:tt)
        @($($struct_constructor:tt)*)
        @($($try_from:tt)*)
        @($($delegate_things: tt)*)
    ) => {
        derive_jack_port_builder!{
            @inner
            $buffer_name
            $builder_name
            $(#[$local_meta:meta])*
            @($($global_tail)*)
            @($process_scope, $self_)
            @($($struct_constructor)* $field_name : <&'static mut dyn CoIterator<Item = Timed<RawMidiEvent>> as $crate::backend::jack_backend::JackBuilder>::Port,)
            @($($try_from)* ($field_name, <&'static mut dyn CoIterator<Item = Timed<RawMidiEvent>> as $crate::backend::jack_backend::JackBuilder>::Port))
            @($($delegate_things)* ($field_name, temp))
        }
    };
}

pub struct JackHandler<B, P> {
    pub builder: B, // TODO: remove the visibility of this?
    pub plugin: P,
}

pub struct PortWrapper<P> {
    inner: P,
}

impl<P> Default for PortWrapper<P>
where
    P: Default,
{
    fn default() -> Self {
        Self {
            inner: P::default(),
        }
    }
}

impl<P> PortWrapper<P> {
    pub fn new(p: P) -> Self {
        Self { inner: p }
    }
}

impl<P> Deref for PortWrapper<P> {
    type Target = P;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> PortWrapper<Port<T>>
where
    T: PortSpec + Default,
{
    pub fn from_client(client: &Client, port_name: &str) -> Result<Self, Error> {
        Ok(Self {
            inner: client.register_port(port_name, T::default())?,
        })
    }
}

impl PortWrapper<Port<AudioIn>> {
    pub fn build<'a>(&'a self, process_scope: &'a ProcessScope) -> &'a [f32] {
        self.inner.as_slice(process_scope)
    }
}

impl<'a> MyInto<&'a [f32]> for &'a [f32] {
    fn my_into(self) -> &'a [f32] {
        self
    }
}

impl PortWrapper<Port<AudioOut>> {
    pub fn build<'a>(&'a mut self, process_scope: &'a ProcessScope) -> &'a mut [f32] {
        self.inner.as_mut_slice(process_scope)
    }
}

impl<'a> MyInto<&'a mut [f32]> for &'a mut [f32] {
    fn my_into(self) -> &'a mut [f32] {
        self
    }
}

impl PortWrapper<Port<MidiIn>> {
    pub fn build<'a, T>(
        &'a mut self,
        process_scope: &'a ProcessScope,
    ) -> impl Iterator<Item = T> + 'a
    where
        T: TryFrom<RawMidi<'a>>,
    {
        self.inner
            .iter(process_scope)
            .filter_map(|e| T::try_from(e).ok())
    }
}

/*
impl<'a> MyInto<&'a mut dyn Iterator<Item = Timed<RawMidiEvent>>> for &'a mut MidiIter<'a> {
    fn my_into(self) -> &'a mut dyn Iterator<Item = Timed<RawMidiEvent>> {
        self.filter_map(|e| Timed::<RawMidiEvent>::try_from(e).ok())
    }
}

 */

impl PortWrapper<Port<MidiOut>> {
    pub fn build<'a>(&'a mut self, process_scope: &'a ProcessScope) -> MidiWriter<'a> {
        self.inner.writer(process_scope)
    }
}

impl<P> DerefMut for PortWrapper<P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<B, P> ProcessHandler for JackHandler<B, P>
where
    for<'a> B: DelegateHandling<P, (&'a Client, &'a ProcessScope), Output = Control>,
    B: Send,
    P: Send,
{
    fn process(&mut self, client: &Client, process_scope: &ProcessScope) -> Control {
        self.builder
            .delegate_handling(&mut self.plugin, (client, process_scope))
    }
}

pub trait JackBuilder {
    type Port;
}

impl JackBuilder for &'static mut dyn Iterator<Item = Timed<RawMidiEvent>> {
    type Port = PortWrapper<Port<MidiIn>>;
}

impl JackBuilder for &'static mut dyn CoIterator<Item = Timed<RawMidiEvent>> {
    type Port = PortWrapper<Port<MidiOut>>;
}

impl JackBuilder for &'static [f32] {
    type Port = PortWrapper<Port<AudioIn>>;
}

impl JackBuilder for &'static mut [f32] {
    type Port = PortWrapper<Port<AudioOut>>;
}

fn plugtestje<'a>(port: &'a mut dyn Iterator<Item = Timed<RawMidiEvent>>) {}

fn testje<'a>(
    port: &'a Port<MidiIn>,
    process_scope: &'a ProcessScope,
) -> impl Iterator<Item = Timed<RawMidiEvent>> + 'a {
    port.iter(process_scope)
        .filter_map(|e| <Timed<RawMidiEvent>>::try_from(e).ok())
}

fn testje2<'a>(port: &'a Port<MidiIn>, process_scope: &'a ProcessScope) {
    let mut x = testje(port, process_scope);
    plugtestje(x.my_into());
}

pub trait MyInto<T> {
    fn my_into(self) -> T;
}

impl<'a, X> MyInto<&'a mut dyn Iterator<Item = <X as Iterator>::Item>> for &'a mut X
where
    X: Iterator,
{
    fn my_into(self) -> &'a mut dyn Iterator<Item = <X as Iterator>::Item> {
        self
    }
}
