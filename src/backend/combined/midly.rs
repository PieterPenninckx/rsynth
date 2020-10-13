use super::MICROSECONDS_PER_SECOND;
use crate::event::{DeltaEvent, RawMidiEvent};
pub use midly::Event;
use midly::{EventKind, Header, MetaMessage, Timing};

const SECONDS_PER_MINUTE: u64 = 60;
const MICROSECONDS_PER_MINUTE: u64 = SECONDS_PER_MINUTE * MICROSECONDS_PER_SECOND;
const DEFAULT_BEATS_PER_MINUTE: u64 = 120;

/// Read from midi events as parsed by the `midly` crate.
pub struct MidlyMidiReader<'v, 'a> {
    events: &'v [Event<'a>],
    index: usize,
    current_tempo_in_micro_seconds_per_beat: f64,
    ticks_per_beat: f64,
}

impl<'v, 'a> MidlyMidiReader<'v, 'a> {
    fn ticks_per_microsecond(&self) -> f64 {
        self.ticks_per_beat / self.current_tempo_in_micro_seconds_per_beat
    }

    /// Create a new `MidlyMidiReader`.
    pub fn new(header: Header, events: &'v [Event<'a>]) -> Self {
        Self {
            events,
            index: 0,
            current_tempo_in_micro_seconds_per_beat: (MICROSECONDS_PER_MINUTE
                / DEFAULT_BEATS_PER_MINUTE)
                as f64,
            ticks_per_beat: match header.timing {
                Timing::Metrical(t) => t.as_int() as f64,
                Timing::Timecode(_, _) => unimplemented!(),
            },
        }
    }
}

impl<'e, 'a> Iterator for MidlyMidiReader<'e, 'a> {
    type Item = DeltaEvent<RawMidiEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut microseconds_since_previous_event = 0.0;
        while let Some(event) = self.events.get(self.index) {
            microseconds_since_previous_event +=
                (event.delta.as_int() as f64) / self.ticks_per_microsecond();
            if let EventKind::Meta(MetaMessage::Tempo(t)) = event.kind {
                self.current_tempo_in_micro_seconds_per_beat = t.as_int() as f64;
            }
            if let EventKind::Midi { .. } = event.kind {
                let mut raw_data: [u8; 3] = [0, 0, 0];
                let mut slice = &mut raw_data[0..3];
                event
                    .kind
                    .write(&mut None, &mut slice)
                    .expect("Unexpected error when writing to memory.");
                // The slice is updated to point to the not-yet-overwritten bytes.
                let number_of_bytes = 3 - slice.len();
                let raw_midi_event = RawMidiEvent::new(&raw_data[0..number_of_bytes]);
                return Some(DeltaEvent {
                    microseconds_since_previous_event: microseconds_since_previous_event as u64,
                    event: raw_midi_event,
                });
            }
            self.index += 1;
        }
        return None;
    }
}

pub struct MidlyMidiWriter {}
