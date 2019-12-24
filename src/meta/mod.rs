//! Example
//! -------
//! ```
//! use rsynth::meta::{Meta, MetaData, InOut};
//! struct MyPlugin {
//!     meta: MetaData<&'static str, &'static str, &'static str>
//! /* ... */
//! }
//!
//! impl MyPlugin {
//!     pub fn new() -> Self {
//!         Self {
//!             meta: MetaData {
//!                 general_meta: unimplemented!(),
//!                 audio_port_meta: InOut {
//!                     inputs: vec![unimplemented!()],
//!                     outputs: vec![unimplemented!()],
//!                 },
//!                 midi_port_meta: InOut {
//!                     inputs: vec![unimplemented!()],
//!                     outputs: vec![unimplemented!()],
//!                 },
//!             }
//!         }
//!     }
//! }
//!
//! impl Meta for MyPlugin {
//!     type MetaData = MetaData<&'static str, &'static str, &'static str>;
//!     fn meta(&self) -> &Self::MetaData {
//!         &self.meta
//!     }
//! }
//! ```

pub trait Meta {
    type MetaData;
    fn meta(&self) -> &Self::MetaData;
}

pub trait General {
    type GeneralData;
    fn general(&self) -> &Self::GeneralData;
}

pub trait Name {
    fn name(&self) -> &str;
}

impl Name for String {
    fn name(&self) -> &str {
        self
    }
}

impl Name for &'static str {
    fn name(&self) -> &str {
        self
    }
}

pub trait Port<T> {
    type PortData;
    fn in_ports(&self) -> &[Self::PortData];
    fn out_ports(&self) -> &[Self::PortData];
}

/// A "marker" struct
pub struct MidiPort;
/// A "marker" struct
pub struct AudioPort;

pub struct MetaData<G, AP, MP> {
    pub general_meta: G,
    pub audio_port_meta: InOut<AP>,
    pub midi_port_meta: InOut<MP>,
}

pub struct InOut<T> {
    pub inputs: Vec<T>,
    pub outputs: Vec<T>,
}

impl<G, AP, MP> General for MetaData<G, AP, MP> {
    type GeneralData = G;
    fn general(&self) -> &G {
        &self.general_meta
    }
}

impl<G, AP, MP> Port<AudioPort> for MetaData<G, AP, MP> {
    type PortData = AP;
    fn in_ports(&self) -> &[AP] {
        self.audio_port_meta.inputs.as_ref()
    }

    fn out_ports(&self) -> &[AP] {
        self.audio_port_meta.outputs.as_ref()
    }
}

impl<G, AP, MP> Port<MidiPort> for MetaData<G, AP, MP> {
    type PortData = MP;

    fn in_ports(&self) -> &[MP] {
        self.midi_port_meta.inputs.as_ref()
    }

    fn out_ports(&self) -> &[MP] {
        self.midi_port_meta.outputs.as_ref()
    }
}
