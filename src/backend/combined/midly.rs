//! Read midi files.
use super::MICROSECONDS_PER_SECOND;
use crate::event::{DeltaEvent, RawMidiEvent};

/// Re-exports from the `midly` crate.
pub mod midly {
    pub use midly::*;
}

use self::midly::Header;
use self::midly::Timing;
use self::midly::TrackEvent;
#[cfg(test)]
use self::midly::{
    num::{u15, u24, u28, u4, u7},
    Format, MidiMessage,
};
use self::midly::{MetaMessage, TrackEventKind};
use std::convert::TryFrom;

const SECONDS_PER_MINUTE: u64 = 60;
const MICROSECONDS_PER_MINUTE: u64 = SECONDS_PER_MINUTE * MICROSECONDS_PER_SECOND;
const DEFAULT_BEATS_PER_MINUTE: u64 = 120;

/// Read from midi events as parsed by the `midly` crate.
pub struct MidlyMidiReader<'v, 'a> {
    events: &'v [TrackEvent<'a>],
    event_index: usize,
    current_tempo_in_micro_seconds_per_beat: f64,
    ticks_per_beat: f64,
}

impl<'v, 'a> MidlyMidiReader<'v, 'a> {
    fn ticks_per_microsecond(&self) -> f64 {
        self.ticks_per_beat / self.current_tempo_in_micro_seconds_per_beat
    }

    /// Create a new `MidlyMidiReader`.
    pub fn new(header: Header, events: &'v [TrackEvent<'a>]) -> Self {
        Self {
            events,
            event_index: 0,
            current_tempo_in_micro_seconds_per_beat: (MICROSECONDS_PER_MINUTE as f64
                / DEFAULT_BEATS_PER_MINUTE as f64),
            ticks_per_beat: match header.timing {
                Timing::Metrical(t) => t.as_int() as f64,
                Timing::Timecode(_, _) => unimplemented!(),
            },
        }
    }
}

#[test]
fn ticks_per_microsecond_works() {
    // 1 beat per second
    let mr = MidlyMidiReader {
        events: &[],
        event_index: 0,
        current_tempo_in_micro_seconds_per_beat: 1000_000.0,
        ticks_per_beat: 100.0 * 1000_000.0,
    };
    assert_eq!(mr.ticks_per_microsecond(), 100.0);
}

#[test]
fn new_works() {
    let header = Header {
        format: Format::SingleTrack,
        timing: Timing::Metrical(u15::from(12345)),
    };
    let mr = MidlyMidiReader::new(header, &[]);
    assert_eq!(mr.event_index, 0);
    assert_eq!(mr.ticks_per_beat, 12345.0);
    // 120 beats per minute
    // = 120 beats per 60 seconds
    // = 120 beats per 60 000 000 microseconds
    // so the tempo is
    //   60 000 000 / 120 beats per microsecond
    //   = 10 000 000 / 20 beats per microsecond
    //   =    500 000 beats per microsecond
    assert_eq!(mr.current_tempo_in_micro_seconds_per_beat, 500000.0);
}

impl<'e, 'a> Iterator for MidlyMidiReader<'e, 'a> {
    type Item = DeltaEvent<RawMidiEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut microseconds_since_previous_event = 0.0;
        while let Some(event) = self.events.get(self.event_index) {
            self.event_index += 1;
            microseconds_since_previous_event +=
                (event.delta.as_int() as f64) / self.ticks_per_microsecond();
            if let TrackEventKind::Meta(MetaMessage::Tempo(new_tempo_in_microseconds_per_beat)) =
                event.kind
            {
                self.current_tempo_in_micro_seconds_per_beat =
                    new_tempo_in_microseconds_per_beat.as_int() as f64;
            }
            if let TrackEventKind::Midi { .. } = event.kind {
                let raw_midi_event = RawMidiEvent::try_from(event.kind).ok()?;
                return Some(DeltaEvent {
                    microseconds_since_previous_event: microseconds_since_previous_event as u64,
                    event: raw_midi_event,
                });
            }
        }
        return None;
    }
}

#[test]
fn iterator_correctly_returns_one_event() {
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
    let header = Header {
        timing: Timing::Metrical(u15::from(ticks_per_beat)),
        format: Format::SingleTrack,
    };
    let mut mr = MidlyMidiReader::new(header, &events);
    let observed = mr.next().expect("MidlyMidiReader should return one event.");
    assert_eq!(observed.microseconds_since_previous_event, 1000000);
    assert_eq!(mr.next(), None);
}

#[cfg(test)]
fn iterator_correctly_returns_two_events() {
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
    let mut mr = MidlyMidiReader::new(header, &events);
    let observed = mr.next().expect("MidlyMidiReader should return one event.");
    assert_eq!(observed.microseconds_since_previous_event, 1000000);
    let observed = mr
        .next()
        .expect("MidlyMidiReader should return a second event.");
    assert_eq!(observed.microseconds_since_previous_event, 1000000);
    assert_eq!(mr.next(), None);
}
