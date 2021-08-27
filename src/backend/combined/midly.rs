//! Read midi files.
use crate::event::{DeltaEvent, RawMidiEvent};

/// Re-exports from the `midly` crate.
pub mod midly_0_5 {
    pub use midly_0_5::*;
}

use self::midly_0_5::Timing;
#[cfg(test)]
use self::midly_0_5::{
    num::{u15, u24, u28, u4, u7},
    Format, Header, MidiMessage, Track, TrackEvent,
};
