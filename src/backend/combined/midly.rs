//! Read midi files.
use super::MICROSECONDS_PER_SECOND;
use crate::event::{DeltaEvent, Indexed, RawMidiEvent, TimeStretcher};

/// Re-exports from the `midly` crate.
pub mod midly_0_5 {
    pub use midly_0_5::*;
}

use self::midly_0_5::Timing;
use self::midly_0_5::{
    live::LiveEvent, num::u28, Arena, Fps, Header, MetaMessage, Track, TrackEvent, TrackEventKind,
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
use std::iter::FromIterator;
use std::num::{NonZeroU64, TryFromIntError};

const SECONDS_PER_MINUTE: u64 = 60;
const MICROSECONDS_PER_MINUTE: u64 = SECONDS_PER_MINUTE * MICROSECONDS_PER_SECOND;
const DEFAULT_BEATS_PER_MINUTE: u64 = 120;

/// Create an iterator over all the tracks, merged.
/// The item has type `(u64, usize, TrackEventKind)`,
/// where the first element of the triple is the timing of the event relative to the beginning
/// of the tracks, in ticks,
/// the second item of the triple is the track index and the last item is the event itself.
///
/// # Example
/// ```
/// use rsynth::backend::combined::midly::{
///     merge_tracks,
///     midly_0_5::{
///         MidiMessage,
///         TrackEvent,
///         TrackEventKind,
///         num::{u15, u4, u7, u28}
///     }
/// };
///
/// // Create a note on event with the given channel
/// fn note_on_with_channel(channel: u8) -> TrackEventKind<'static> {
///     // ...
/// #     TrackEventKind::Midi {
/// #         channel: u4::new(channel),
/// #         message: MidiMessage::NoteOn {
/// #             key: u7::new(1),
/// #             vel: u7::new(1),
/// #         },
/// #     }
/// }
///
/// // Create a note on event with the given delta and channel.
/// fn note_on_with_delta_and_channel(delta: u32, channel: u8) -> TrackEvent<'static> {
///     // ...
/// #     TrackEvent {
/// #        delta: u28::new(delta),
/// #        kind: note_on_with_channel(channel),
/// #    }
/// }
///
/// let tracks = vec![
///     vec![
///         note_on_with_delta_and_channel(2, 0),
///         note_on_with_delta_and_channel(100, 1),
///     ],
///     vec![note_on_with_delta_and_channel(30, 2)],
/// ];
/// let result: Vec<_> = merge_tracks(&tracks[..]).collect();
/// assert_eq!(
///     result,
///     vec![
///         (2, 0, note_on_with_channel(0)),
///         (30, 1, note_on_with_channel(2)),
///         (102, 0, note_on_with_channel(1)),
///     ]
/// )
/// ```
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

#[test]
fn merge_tracks_works() {
    fn kind(channel: u8) -> TrackEventKind<'static> {
        TrackEventKind::Midi {
            channel: u4::new(channel),
            message: MidiMessage::NoteOn {
                key: u7::new(1),
                vel: u7::new(1),
            },
        }
    }
    fn track_event(delta: u32, channel: u8) -> TrackEvent<'static> {
        TrackEvent {
            delta: u28::new(delta),
            kind: kind(channel),
        }
    }

    let tracks = vec![
        vec![track_event(1, 0), track_event(2, 1), track_event(4, 2)],
        vec![track_event(2, 3), track_event(2, 4), track_event(5, 5)],
    ];
    let result: Vec<_> = merge_tracks(&tracks[..]).collect();
    assert_eq!(
        result,
        vec![
            (1, 0, kind(0)),
            (2, 1, kind(3)),
            (3, 0, kind(1)),
            (4, 1, kind(4)),
            (7, 0, kind(2)),
            (9, 1, kind(5))
        ]
    )
}

pub struct TrackMerger {}

#[derive(Debug)]
#[non_exhaustive]
pub enum TimeConversionCreateError {
    /// The header indicates that there are zero ticks per beat.
    ZeroTicksPerBeatNotSupported,
    /// The header indicates that there are zero ticks per frame.
    ZeroTicksPerFrameNotSupported,
}

impl Display for TimeConversionCreateError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            TimeConversionCreateError::ZeroTicksPerBeatNotSupported => {
                write!(f, "zero ticks per beat is not supported")
            }
            TimeConversionCreateError::ZeroTicksPerFrameNotSupported => {
                write!(f, "zero ticks per frame is not supported")
            }
        }
    }
}

impl Error for TimeConversionCreateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

/// Convert timings of [`TrackEventKind`] from ticks to microseconds.
pub struct ConvertTicksToMicroseconds {
    time_stretcher: TimeStretcher,
    ticks_per_beat: Option<NonZeroU64>,
}

impl ConvertTicksToMicroseconds {
    /// Create a new `ConvertTicksToMicrosecond` with the given header.
    ///
    /// The parameter `header` is used to determine the meaning of "tick", since this is stored
    /// in the header in a midi file.  
    pub fn new(header: Header) -> Result<Self, TimeConversionCreateError> {
        let time_stretcher;
        let ticks_per_beat;
        use TimeConversionCreateError::*;
        match header.timing {
            Timing::Metrical(t) => {
                let tpb = NonZeroU64::new(t.as_int() as u64).ok_or(ZeroTicksPerBeatNotSupported)?;
                time_stretcher =
                    TimeStretcher::new(MICROSECONDS_PER_MINUTE / DEFAULT_BEATS_PER_MINUTE, tpb);
                ticks_per_beat = Some(tpb);
            }
            Timing::Timecode(Fps::Fps29, ticks_per_frame) => {
                // Frames per second = 30 / 1.001 = 30000 / 1001
                // microseconds = ticks * microseconds_per_second / (ticks_per_frame * frames_per_second) ;
                time_stretcher = TimeStretcher::new(
                    MICROSECONDS_PER_SECOND * 1001,
                    NonZeroU64::new((ticks_per_frame as u64) * 30000)
                        .ok_or(ZeroTicksPerFrameNotSupported)?,
                );
                ticks_per_beat = None;
            }
            Timing::Timecode(fps, ticks_per_frame) => {
                // microseconds = ticks * microseconds_per_second / (ticks_per_frame * frames_per_second) ;
                time_stretcher = TimeStretcher::new(
                    MICROSECONDS_PER_SECOND,
                    NonZeroU64::new((fps.as_int() as u64) * (ticks_per_frame as u64))
                        .ok_or(ZeroTicksPerFrameNotSupported)?,
                );
                ticks_per_beat = None;
            }
        }
        Ok(Self {
            ticks_per_beat,
            time_stretcher,
        })
    }

    /// Return the time of the event, in microseconds, relative to the beginning of the track.
    ///
    /// # Parameters
    /// `ticks`: the absolute time, in ticks, relative to the beginning of the track.
    /// It is assumed that this only increases with subsequent calls to this method.
    /// `event`: the event.
    ///
    /// # Return value
    /// The absolute time, in microseconds, relative to the beginning of the track.
    pub fn convert<'a>(&mut self, ticks: u64, event: &TrackEventKind<'a>) -> u64 {
        let new_factor = if let Some(ticks_per_beat) = self.ticks_per_beat {
            if let TrackEventKind::Meta(MetaMessage::Tempo(tempo)) = event {
                Some((tempo.as_int() as u64, ticks_per_beat))
            } else {
                None
            }
        } else {
            None
        };
        self.time_stretcher.stretch(ticks, new_factor)
    }
}

#[test]
pub fn convert_ticks_to_microsoconds_works_with_one_event() {
    // 120 beats per minute
    // = 120 beats per 60 seconds
    // = 120 beats per 60 000 000 microseconds
    // so the tempo is
    //   60 000 000 / 120 microseconds per beat
    //   = 10 000 000 / 20 microseconds per beat
    //   =    500 000 microseconds per beat
    let tempo_in_microseconds_per_beat = 500000;
    let ticks_per_beat = 32;
    // One event after 1 second.
    // One second corresponds to two beats, so to 64 ticks.
    let event_time_in_ticks: u64 = 64;
    let header = Header {
        timing: Timing::Metrical(u15::from(ticks_per_beat)),
        format: Format::SingleTrack,
    };
    let mut converter =
        ConvertTicksToMicroseconds::new(header).expect("No error expected at this point.");
    let milliseconds = converter.convert(
        0,
        &TrackEventKind::Meta(MetaMessage::Tempo(u24::from(
            tempo_in_microseconds_per_beat,
        ))),
    );
    assert_eq!(milliseconds, 0);
    let milliseconds = converter.convert(
        event_time_in_ticks,
        &TrackEventKind::Midi {
            channel: u4::from(0),
            message: MidiMessage::NoteOn {
                key: u7::from(60),
                vel: u7::from(90),
            },
        },
    );
    assert_eq!(milliseconds, 1000000);
}

/// The error returned by the [`separate_tracks`] function.
#[derive(Debug)]
#[non_exhaustive]
pub enum SeparateTracksError {
    /// The specified time overflows what can be specified.
    TimeOverflow,
    /// Time decreased from one event to a next event.
    TimeCanOnlyIncrease,
}

impl From<TryFromIntError> for SeparateTracksError {
    fn from(_: TryFromIntError) -> Self {
        SeparateTracksError::TimeOverflow
    }
}

impl Display for SeparateTracksError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            SeparateTracksError::TimeOverflow => {
                write!(
                    f,
                    "the time overflows what can be represented in a midi file."
                )
            }
            SeparateTracksError::TimeCanOnlyIncrease => {
                write!(f, "subsequent events with decreasing time found.")
            }
        }
    }
}

impl Error for SeparateTracksError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

/// Write to a track, keeping ... well, keeping "track" of the timing relative to the beginning.
#[derive(Clone)]
struct TrackWriter<'a> {
    track: Track<'a>,
    ticks: u64,
}

impl<'a> TrackWriter<'a> {
    fn new() -> Self {
        TrackWriter {
            track: Vec::new(),
            ticks: 0,
        }
    }

    fn push(&mut self, ticks: u64, kind: TrackEventKind<'a>) -> Result<(), SeparateTracksError> {
        use SeparateTracksError::*;
        if self.ticks > ticks {
            return Err(TimeCanOnlyIncrease);
        }
        let delta = ticks - self.ticks;
        let delta = u28::try_from(u32::try_from(delta)?).ok_or(TimeOverflow)?;
        self.ticks = ticks;
        Ok(self.track.push(TrackEvent { kind, delta }))
    }
}

/// Separate tracks.
pub struct TrackSeparator<'a> {
    tracks: Vec<TrackWriter<'a>>,
}

impl<'a> TrackSeparator<'a> {
    /// Create a new `TrackSeparator`.
    ///
    /// # Example
    /// ```
    /// use rsynth::backend::combined::midly::TrackSeparator;
    /// let track_separator = TrackSeparator::new();
    /// let tracks : Vec<_> = track_separator.collect();
    /// assert!(tracks.is_empty());
    /// ```
    #[inline]
    pub fn new() -> Self {
        TrackSeparator { tracks: Vec::new() }
    }

    /// Create a new `TrackSeparator` from the elements of the given iterator.
    ///
    /// # Example
    /// ```
    /// use rsynth::backend::combined::midly::TrackSeparator;
    /// use rsynth::backend::combined::midly::midly_0_5::{TrackEventKind, TrackEvent};
    /// # use rsynth::backend::combined::midly::midly_0_5::{MidiMessage, num::{u4, u7, u28}};
    ///
    /// fn note_on_with_channel(channel: u8) -> TrackEventKind<'static> {
    ///     // ...
    /// #    TrackEventKind::Midi {
    /// #        channel: u4::new(channel),
    /// #        message: MidiMessage::NoteOn {
    /// #            key: u7::new(1),
    /// #            vel: u7::new(1),
    /// #        },
    /// #    }
    /// }
    ///
    /// fn note_on_with_channel_and_delta_time(delta: u32, channel: u8) -> TrackEvent<'static> {
    ///     // ...
    /// #    TrackEvent {
    /// #        delta: u28::new(delta),
    /// #        kind: note_on_with_channel(channel),
    /// #    }
    /// }
    ///
    /// let events : Vec<(u64, usize, _)>= vec![
    ///     (1, 0, note_on_with_channel(0)),
    ///     (2, 1, note_on_with_channel(3)),
    ///     (3, 0, note_on_with_channel(1)),
    ///     (4, 1, note_on_with_channel(4)),
    ///     (7, 0, note_on_with_channel(2)),
    ///     (9, 1, note_on_with_channel(5)),
    /// ];
    ///
    /// let separated : Vec<_> = TrackSeparator::from_iterator(events.into_iter())
    ///     .expect("No error should occur here.")
    ///     .collect();
    ///
    /// assert_eq!(
    ///     separated,
    ///     vec![
    ///         vec![
    ///             note_on_with_channel_and_delta_time(1, 0),
    ///             note_on_with_channel_and_delta_time(2, 1),
    ///             note_on_with_channel_and_delta_time(4, 2)
    ///         ],
    ///         vec![
    ///             note_on_with_channel_and_delta_time(2, 3),
    ///             note_on_with_channel_and_delta_time(2, 4),
    ///             note_on_with_channel_and_delta_time(5, 5)
    ///         ],
    ///     ]
    /// );
    /// ```
    #[inline]
    pub fn from_iterator<I>(iterator: I) -> Result<Self, SeparateTracksError>
    where
        I: Iterator<Item = (u64, usize, TrackEventKind<'a>)>,
    {
        let mut result = TrackSeparator::new();
        result.extend(iterator)?;
        Ok(result)
    }

    /// Push a new event.
    ///
    /// # Parameters
    /// * `ticks`: the time in midi ticks, relative to the beginning
    ///   of the tracks
    /// * `track_index`: the index of the track to which the event belongs
    /// * `event`: the event
    ///
    /// Create a new `TrackSeparator`.
    ///
    /// # Example
    /// ```
    /// use rsynth::backend::combined::midly::TrackSeparator;
    /// use rsynth::backend::combined::midly::midly_0_5::{TrackEventKind, TrackEvent};
    /// # use rsynth::backend::combined::midly::{
    /// #     midly_0_5::{
    /// #         MidiMessage,
    /// #         num::{u4, u7, u28}
    /// #     }
    /// # };
    ///
    /// // Create a note on event with the given channel
    /// fn note_on_with_channel(channel: u8) -> TrackEventKind<'static> {
    ///     // ...
    /// #     TrackEventKind::Midi {
    /// #         channel: u4::new(channel),
    /// #         message: MidiMessage::NoteOn {
    /// #             key: u7::new(1),
    /// #             vel: u7::new(1),
    /// #         },
    /// #     }
    /// }
    ///
    /// // Create a note on event with the given delta and channel.
    /// fn note_on_with_delta_and_channel(delta: u32, channel: u8) -> TrackEvent<'static> {
    ///     // ...
    /// #     TrackEvent {
    /// #        delta: u28::new(delta),
    /// #        kind: note_on_with_channel(channel),
    /// #    }
    /// }
    /// let mut track_separator = TrackSeparator::new();
    /// track_separator.push(5, 0, note_on_with_channel(0));
    /// track_separator.push(0, 1, note_on_with_channel(1));
    /// track_separator.push(10, 0, note_on_with_channel(2));
    /// let tracks : Vec<_> = track_separator.collect();
    /// assert_eq!(tracks.len(), 2);
    /// assert_eq!(
    ///             tracks[0],
    ///             vec![
    ///                 note_on_with_delta_and_channel(5, 0),
    ///                 note_on_with_delta_and_channel(5, 2)
    ///             ]
    /// );
    /// assert_eq!(tracks[1], vec![note_on_with_delta_and_channel(0, 1)]);
    /// ```
    #[inline]
    pub fn push(
        &mut self,
        ticks: u64,
        track_index: usize,
        event: TrackEventKind<'a>,
    ) -> Result<(), SeparateTracksError> {
        if self.tracks.len() <= track_index {
            self.tracks.resize(track_index + 1, TrackWriter::new());
        }
        self.tracks[track_index].push(ticks, event)
    }

    /// Append all events from an iterator of triples of type `(u64, usize, TrackEventKind)`,
    /// where
    /// * the first item of the triple (of type `u64`) is the time in midi ticks, relative to the beginning
    ///   of the tracks
    /// * the second item of the triple (of type `usize`) is the track index.
    ///   The highest track index determines the length of the resulting `Vec`.
    /// * the last item of the triple is the event itself.
    #[inline]
    pub fn extend<I>(&mut self, iterator: I) -> Result<(), SeparateTracksError>
    where
        I: Iterator<Item = (u64, usize, TrackEventKind<'a>)>,
    {
        for (ticks, track_index, event) in iterator {
            self.push(ticks, track_index, event)?
        }
        Ok(())
    }

    /// Create a collection containing all the tracks.
    pub fn collect<B>(self) -> B
    where
        B: FromIterator<Track<'a>>,
    {
        self.tracks.into_iter().map(|t| t.track).collect()
    }
}
