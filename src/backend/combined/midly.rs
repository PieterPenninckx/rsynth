//! Read midi files.
use super::MICROSECONDS_PER_SECOND;
use crate::event::{DeltaEvent, RawMidiEvent, TimeStretcher};

/// Re-exports from the `midly` crate.
pub mod midly {
    pub use midly::*;
}

use self::midly::Timing;
#[cfg(test)]
use self::midly::{
    num::{u15, u24, u28, u4, u7},
    Format, Header, MidiMessage, Track, TrackEvent,
};
use self::midly::{MetaMessage, TrackEventKind};
use crate::backend::combined::midly::midly::Smf;
use itertools::Itertools;
use std::convert::TryFrom;
use std::num::NonZeroU64;

const SECONDS_PER_MINUTE: u64 = 60;
const MICROSECONDS_PER_MINUTE: u64 = SECONDS_PER_MINUTE * MICROSECONDS_PER_SECOND;
const DEFAULT_BEATS_PER_MINUTE: u64 = 120;

/// Read from midi events as parsed by the `midly` crate.
pub struct MidlyMidiReader<'a, 'b> {
    event_iter: Box<dyn Iterator<Item = (u64, TrackEventKind<'a>)> + 'b>,
    timestretcher: TimeStretcher,
    previous_time_in_microseconds: u64,
    ticks_per_beat: NonZeroU64,
}

impl<'a, 'b> MidlyMidiReader<'a, 'b>
where
    'b: 'a,
{
    /// Create a new `MidlyMidiReader` that will read all tracks together (interleaved).
    pub fn new(smf: &'b Smf<'a>) -> Result<Self, ()> {
        let track_mask: Vec<_> = smf.tracks.iter().map(|_| true).collect();
        Self::new_with_track_mask(smf, &track_mask)
    }

    /// Create a new `MidlyMidiReader` that will read only the masked tracks (interleaved).
    ///
    /// # Parameters
    /// `smf`: the [`Smf`] for reading the midi file
    /// `track_mask`: a slice of booleans, only the tracks that correspond to `true` will be read.
    pub fn new_with_track_mask(smf: &'b Smf<'a>, track_mask: &[bool]) -> Result<Self, ()> {
        let mut event_iter: Box<dyn Iterator<Item = (u64, TrackEventKind<'a>)> + 'b> =
            Box::new(Vec::new().into_iter());
        for (must_include, track) in track_mask.iter().zip(smf.tracks.iter()) {
            if *must_include {
                let mut offset = 0;
                let iter = track.iter().map(move |e| {
                    offset += e.delta.as_int() as u64;
                    (offset, e.kind)
                });
                event_iter = Box::new(event_iter.merge_by(iter, |(t1, _), (t2, _)| t1 < t2));
            }
        }
        let ticks_per_beat = match smf.header.timing {
            Timing::Metrical(t) => NonZeroU64::new(t.as_int() as u64).ok_or(())?,
            Timing::Timecode(_, _) => return Err(()),
        };
        //                   ticks * microseconds_per_beat
        // microseconds = -----------------------------------
        //                   ticks_per_beat
        let timestretcher = TimeStretcher::new(
            MICROSECONDS_PER_MINUTE / DEFAULT_BEATS_PER_MINUTE,
            ticks_per_beat,
        );
        Ok(Self {
            ticks_per_beat,
            event_iter,
            previous_time_in_microseconds: 0,
            timestretcher,
        })
    }
}

impl<'a, 'b> Iterator for MidlyMidiReader<'a, 'b> {
    type Item = DeltaEvent<RawMidiEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (t, event) = self.event_iter.next()?;
            let new_factor = if let TrackEventKind::Meta(MetaMessage::Tempo(tempo)) = event {
                Some((tempo.as_int() as u64, self.ticks_per_beat))
            } else {
                None
            };
            let time = self.timestretcher.stretch(t, new_factor);
            if let TrackEventKind::Midi { .. } = event {
                if let Ok(e) = RawMidiEvent::try_from(event) {
                    let difference = time - self.previous_time_in_microseconds;
                    self.previous_time_in_microseconds = time;
                    return Some(DeltaEvent {
                        microseconds_since_previous_event: difference,
                        event: e,
                    });
                }
            }
        }
    }
}

#[test]
pub fn iterator_correctly_returns_one_event() {
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
    let event_time_in_ticks = 64;
    let events = vec![
        TrackEvent {
            delta: u28::from(0),
            kind: TrackEventKind::Meta(MetaMessage::Tempo(u24::from(
                tempo_in_microseconds_per_beat,
            ))),
        },
        TrackEvent {
            delta: u28::from(event_time_in_ticks),
            kind: TrackEventKind::Midi {
                channel: u4::from(0),
                message: MidiMessage::NoteOn {
                    key: u7::from(60),
                    vel: u7::from(90),
                },
            },
        },
    ];
    let tracks = vec![events];
    let header = Header {
        timing: Timing::Metrical(u15::from(ticks_per_beat)),
        format: Format::SingleTrack,
    };
    let smf = Smf { header, tracks };
    let mut mr = MidlyMidiReader::new(&smf).expect("No errors should occur now.");
    let observed = mr.next().expect("MidlyMidiReader should return one event.");
    assert_eq!(observed.microseconds_since_previous_event, 1000000);
    assert_eq!(mr.next(), None);
}

#[cfg(test)]
pub fn iterator_correctly_returns_two_events() {
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
    let event_delta_time_in_ticks = 64;
    let events = vec![
        TrackEvent {
            delta: u28::from(0),
            kind: TrackEventKind::Meta(MetaMessage::Tempo(u24::from(
                tempo_in_microseconds_per_beat,
            ))),
        },
        TrackEvent {
            delta: u28::from(event_delta_time_in_ticks),
            kind: TrackEventKind::Midi {
                channel: u4::from(0),
                message: MidiMessage::NoteOn {
                    key: u7::from(60),
                    vel: u7::from(90),
                },
            },
        },
        TrackEvent {
            delta: u28::from(event_delta_time_in_ticks),
            kind: TrackEventKind::Midi {
                channel: u4::from(0),
                message: MidiMessage::NoteOn {
                    key: u7::from(60),
                    vel: u7::from(90),
                },
            },
        },
    ];
    let header = Header {
        timing: Timing::Metrical(u15::from(ticks_per_beat)),
        format: Format::SingleTrack,
    };
    let tracks = vec![events];
    let smf = Smf { header, tracks };
    let mut mr = MidlyMidiReader::new(&smf).expect("No errors should occur now");
    let observed = mr.next().expect("MidlyMidiReader should return one event.");
    assert_eq!(observed.microseconds_since_previous_event, 1000000);
    let observed = mr
        .next()
        .expect("MidlyMidiReader should return a second event.");
    assert_eq!(observed.microseconds_since_previous_event, 1000000);
    assert_eq!(mr.next(), None);
}
