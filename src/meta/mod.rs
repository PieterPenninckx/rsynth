//! Mechanisms for defining the meta-data of a plugin or application.
//!
//! `rsynth` uses a hierarchy of different traits that allow your audio application
//! or plug-in to define various aspects of the meta-data.
//!
//! Implementing each of these traits one by one can be rather tedious.
//! For this reason, these traits have blanket impls, so that you only need
//! to implement the [`Meta`] trait and in its implementation, return the
//! meta-data.
//!
//! Example
//! -------
//! ```
//! use rsynth::meta::{Meta, MetaData, InOut};
//! struct MyPlugin {
//!     meta: MetaData<&'static str, &'static str, &'static str>
//!     /* ... */
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

/// Define the meta-data for an application or plug-in.
///
/// See the [module level documentation] for more details.
///
/// [module level documentation]: ./index.html
/// [`MetaData`]: ./struct.MetaData.html
/// [`Meta`]: ./trait.Meta.html
pub trait Meta {
    /// The data-type that represents the meta-data.
    ///
    /// Note
    /// ----
    /// In most cases, the struct [`MetaData`] can be used for this associated type.
    ///
    /// [`MetaData`]: ./struct.MetaData.html
    type MetaData;

    /// Get the meta-data.
    fn meta(&self) -> &Self::MetaData;
}

/// Define meta-data of an application or plugin as a whole.
pub trait General {
    /// The data-type of the general meta-data.
    type GeneralData;
    /// Get the general meta-data.
    fn general(&self) -> &Self::GeneralData;
}

/// Implement this trait to indicate that a type can be used to represent
/// meta-data that contains a name.
pub trait Name {
    /// Get the name.
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

/// Define meta-data for input ports and output ports.
///
/// The type parameter `T` is a dummy type parameter so that meta-data for different types of
/// ports can be defined.
/// Typical values for `T` are [`MidiPort`] and [`AudioPort`].
///
/// Example
/// -------
/// ```
/// use rsynth::meta::{Port, MidiPort, AudioPort};
/// struct MyMetaData {
///     audio_input_port_names: Vec<String>,
///     audio_output_port_names: Vec<String>,
///     midi_input_port_names: Vec<String>,
///     midi_output_port_names: Vec<String>,
/// }
///
/// impl Port<AudioPort> for MyMetaData {
///     type PortData = String;
///     fn in_ports(&self) -> &[Self::PortData] {
///         self.audio_input_port_names.as_slice()
///     }
///     fn out_ports(&self) -> &[Self::PortData] {
///         self.audio_output_port_names.as_slice()
///     }
/// }
///
/// impl Port<MidiPort> for MyMetaData {
///     type PortData = String;
///     fn in_ports(&self) -> &[Self::PortData] {
///         self.audio_input_port_names.as_slice()
///     }
///     fn out_ports(&self) -> &[Self::PortData] {
///         self.audio_output_port_names.as_slice()
///     }
/// }
/// ```
///
/// Note
/// ----
/// For most use cases, you can use the pre-defined [`MetaData`] struct, which already implements
/// `Port<MidiPort>` and `Port<AudioPort>`.
///
/// [`MidiPort`]: ./struct.MidiPort.html
/// [`AudioPort`]: ./struct.AudioPort.html
/// [`MetaData`]: ./struct.MetaData.html
pub trait Port<T> {
    type PortData;
    fn in_ports(&self) -> &[Self::PortData];
    fn out_ports(&self) -> &[Self::PortData];
}

/// A "marker" struct to be used as a type parameter for the [`Port`] trait, indicating
/// that this implementation of [`Port`] defines meta-data for an audio port.
///
/// [`Port']: ./trait.Port.html
pub struct AudioPort;

/// A "marker" struct to be used as a type parameter for the [`Port`] trait, indicating
/// that this implementation of [`Port`] defines meta-data for a midi port.
///
/// [`Port']: ./trait.Port.html
pub struct MidiPort;

/// Represents general-purpose meta-data of an audio application or plugin.
///
/// See the [module level documentation] for an example.
///
/// [module level documentation]: ./index.html
pub struct MetaData<G, AP, MP> {
    /// The meta-data about the application or plugin as a whole.
    pub general_meta: G,
    /// Meta-data about the audio ports.
    pub audio_port_meta: InOut<AP>,
    /// Meta-data about the midi ports.
    pub midi_port_meta: InOut<MP>,
}

/// Represents meta-data about a input and output ports.
///
/// See the documentation of the [`MetaData`] struct for more information.
///
/// [`MetaData`]: ./struct.MetaData.html
pub struct InOut<T> {
    /// Meta-data of the input ports.
    pub inputs: Vec<T>,
    /// Meta-data of the output ports.
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
