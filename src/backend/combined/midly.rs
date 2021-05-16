//! Read midi files.
use super::MICROSECONDS_PER_SECOND;
use crate::event::{DeltaEvent, Indexed, RawMidiEvent, TimeStretcher};

/// Re-exports from the `midly` crate.
pub mod midly_0_5 {
    pub use midly_0_5::*;
}

use self::midly_0_5::Timing;
use self::midly_0_5::{
    live::LiveEvent, num::u28, Arena, Header, MetaMessage, Track, TrackEvent, TrackEventKind,
};
#[cfg(test)]
use self::midly_0_5::{
    num::{u15, u24, u4, u7},
    Format, MidiMessage,
};
use crate::backend::combined::midly::midly_0_5::Smf;
use itertools::Itertools;
use midi_consts::channel_event::NOTE_ON;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::num::{NonZeroU64, TryFromIntError};

const SECONDS_PER_MINUTE: u64 = 60;
const MICROSECONDS_PER_MINUTE: u64 = SECONDS_PER_MINUTE * MICROSECONDS_PER_SECOND;
const DEFAULT_BEATS_PER_MINUTE: u64 = 120;

/// Create an iterator over all the tracks, merged.
/// The item has type `(u64, usize, TrackEventKind)`,
/// where the first element of the triple is the timing of the index, in ticks,
/// the second item of the triple is the track index and the last item is the event itself.
pub fn merge_tracks<'a, 'b>(
    tracks: &'b [Track<'a>],
) -> impl Iterator<Item = (u64, usize, TrackEventKind<'a>)> + 'b
where
    'b: 'a,
{
    let mut track_index = 0;
    tracks
        .iter()
        .map(|t| {
            let mut offset = 0;
            let result = t.iter().map(move |e| {
                offset += e.delta.as_int() as u64;
                (offset, track_index, e.kind)
            });
            track_index += 1;
            result
        })
        .kmerge_by(|(t1, _, _), (t2, _, _)| t1 < t2)
}

enum TrackPushError {
    TimingUnderflow,
    TimingOverflow,
}

impl From<TryFromIntError> for TrackPushError {
    fn from(_: TryFromIntError) -> Self {
        TrackPushError::TimingOverflow
    }
}

#[derive(Debug)]
pub enum TrackWritingError {
    TrackIndexLargerThanNumberOfTracks {
        track_index: usize,
        number_of_tracks: usize,
    },
    TimingOverflow,
    TimingCanOnlyIncrease,
}

impl From<TrackPushError> for TrackWritingError {
    fn from(tpe: TrackPushError) -> Self {
        match tpe {
            TrackPushError::TimingUnderflow => TrackWritingError::TimingCanOnlyIncrease,
            TrackPushError::TimingOverflow => TrackWritingError::TimingOverflow,
        }
    }
}

impl Display for TrackWritingError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            TrackWritingError::TrackIndexLargerThanNumberOfTracks {
                track_index,
                number_of_tracks,
            } => write!(
                f,
                "Track index ({}) is larger than the number of tracks ({})",
                track_index, number_of_tracks
            ),
            TrackWritingError::TimingOverflow => {
                write!(f, "The specified timing overflows what can be specified.")
            }
            TrackWritingError::TimingCanOnlyIncrease => {
                write!(f, "Events with decreasing timing found.")
            }
        }
    }
}

impl Error for TrackWritingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

struct TrackWriter<'a> {
    track: Track<'a>,
    ticks: u64,
}

impl<'a> TrackWriter<'a> {
    pub fn new() -> Self {
        TrackWriter {
            track: Vec::new(),
            ticks: 0,
        }
    }

    pub fn push(&mut self, ticks: u64, kind: TrackEventKind<'a>) -> Result<(), TrackPushError> {
        if self.ticks > ticks {
            return Err(TrackPushError::TimingUnderflow);
        }
        let delta = ticks - self.ticks;
        let delta = u28::try_from(u32::try_from(delta)?).ok_or(TrackPushError::TimingOverflow)?;
        self.ticks += ticks;
        Ok(self.track.push(TrackEvent { kind, delta: delta }))
    }
}

pub fn split_tracks<'a, I>(
    iterator: I,
    number_of_tracks: usize,
) -> Result<Vec<Track<'a>>, TrackWritingError>
where
    I: Iterator<Item = (u64, usize, TrackEventKind<'a>)>,
{
    use TrackWritingError::*;
    let mut tracks = Vec::with_capacity(number_of_tracks);
    for _ in 0..number_of_tracks {
        tracks.push(TrackWriter::new());
    }
    for (timing, track_index, event) in iterator {
        let track = tracks
            .get_mut(track_index)
            .ok_or(TrackIndexLargerThanNumberOfTracks {
                track_index,
                number_of_tracks,
            })?;
        track.push(timing, event)?;
    }
    Ok(tracks.into_iter().map(|t| t.track).collect())
}
