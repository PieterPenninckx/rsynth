//! Combine different back-ends for audio input, audio output and midi input,
//! mostly for offline rendering and testing.
//!
//! Support is only enabled if `rsynth` is compiled with the `backend-combined`
//! feature, see [the cargo reference] for more information on setting cargo features.
//!
//! The [`run`] function can be used to run a plugin and read audio and midi from the
//! inputs and write audio and midi to the outputs.
//!
//! Currently, the following inputs and outputs are available:
//!
//! * Dummy: [`AudioDummy`]: dummy audio input (generates silence) and output and [`MidiDummy`]: dummy midi input (generates no events) and output
//! * Hound: [`HoundAudioReader`] and [`HoundAudioWriter`]: read and write `.wav` files (behind the "backend-combined-hound" feature)
//! * Midly: [`MidlyMidiReader`]: read `.mid` files (behind the "backend-combined-midly-0-5" feature)
//! * Memory: [`AudioBufferReader`] and [`AudioBufferWriter`]: read and write audio from memory
//! * Testing: [`TestAudioReader`] and [`TestAudioWriter`]: audio input and output, to be used in tests
//!
//! Note that, when compiled with the `backend-combined-wav` feature,
//! [`AudioChunkReader`] implements `From<(Header, BitDepth)>`
//! (`Header` and `BitDepth` are from the `wav` crate) to ease integration with the `wav` crate.
//!
//! [`AudioDummy`]: ./dummy/struct.AudioDummy.html
//! [`MidiDummy`]: ./dummy/struct.MidiDummy.html
//! [`HoundAudioReader`]: ./hound/struct.HoundAudioReader.html
//! [`HoundAudioWriter`]: ./hound/struct.HoundAudioWriter.html
//! [`MidlyMidiReader`]: ./midly/struct.MidlyMidiReader.html
//! [`TestAudioReader`]: ./struct.TestAudioReader.html
//! [`TestAudioWriter`]: ./struct.TestAudioWriter.html
//! [`AudioBufferReader`]: ./memory/struct.AudioBufferReader.html
//! [`AudioBufferWriter`]: ./memory/struct.AudioBufferWriter.html
//! [`run`]: ./fn.run.html
//! [the cargo reference]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-features-section
//! [`AudioChunkReader`]: ./memory/struct.AudioChunkReader.html

use crate::backend::{HostInterface, Stop};
use crate::event::{DeltaEvent, EventHandler, Indexed, RawMidiEvent, Timed};
use event_queue::{AlwaysInsertNewAfterOld, EventQueue};
use num_traits::Zero;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

pub mod dummy;
#[cfg(feature = "backend-combined-hound")]
pub mod hound;
pub mod memory;
#[cfg(feature = "backend-combined-midly-0-5")]
pub mod midly;

/// The error type that represents the errors you can get from the [`run`] function.
///
/// [`run`]: ./fn.run.html
#[derive(Debug)]
pub enum CombinedError<AudioInErr, AudioOutErr> {
    /// An error occurred when reading the audio.
    AudioInError(AudioInErr),
    /// An error occurred when writing the audio.
    AudioOutError(AudioOutErr),
}

impl<AudioInErr, AudioOutErr> Display for CombinedError<AudioInErr, AudioOutErr>
where
    AudioInErr: Display,
    AudioOutErr: Display,
{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            CombinedError::AudioInError(ref e) => write!(f, "Audio in error: {}", e),
            CombinedError::AudioOutError(ref e) => write!(f, "Audio out error: {}", e),
        }
    }
}

impl<AudioInErr, AudioOutErr> Error for CombinedError<AudioInErr, AudioOutErr>
where
    AudioInErr: Error,
    AudioOutErr: Error,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CombinedError::AudioInError(ref e) => e.source(),
            CombinedError::AudioOutError(ref e) => e.source(),
        }
    }
}
