use super::{DeltaEvent, MidiReader, MICROSECONDS_PER_SECOND};
use crate::event::RawMidiEvent;
use rimd::{Event, MetaCommand, MetaEvent, Track, TrackEvent, SMF};

const SECONDS_PER_MINUTE: u64 = 60;
const MICROSECONDS_PER_MINUTE: u64 = SECONDS_PER_MINUTE * MICROSECONDS_PER_SECOND;

#[derive(Debug)]
pub enum MidiHandleError {
    NotOneTrack { number_of_tracks: usize },
    TimeDivisionNotSupported,
    TempoSetMoreThanOnce,
    TempoSetParseError,
}

const DEFAULT_BEATS_PER_MINUTE: u64 = 120;

pub struct RimdMidiReader<'a> {
    track_iterator: std::slice::Iter<'a, TrackEvent>,
    current_tempo_in_micro_seconds_per_beat: f64,
    ticks_per_beat: f64,
}

impl<'a> RimdMidiReader<'a> {
    pub fn ticks_per_microsecond(&self) -> f64 {
        self.ticks_per_beat / self.current_tempo_in_micro_seconds_per_beat
    }

    pub fn new(input_file: &'a SMF, track_index: usize) -> Self {
        if input_file.tracks.len() < track_index {
            unimplemented!("Implement better error handling when the track index cannot be found");
        }
        if input_file.division < 0 {
            unimplemented!("Support 'negative' time division");
        }
        let ticks_per_beat = input_file.division as f64;
        Self {
            track_iterator: input_file.tracks[track_index].events.iter(),
            current_tempo_in_micro_seconds_per_beat: (MICROSECONDS_PER_MINUTE
                / DEFAULT_BEATS_PER_MINUTE)
                as f64,
            ticks_per_beat,
        }
    }
}

impl<'a> MidiReader for RimdMidiReader<'a> {
    fn read_event(&mut self) -> Option<DeltaEvent<RawMidiEvent>> {
        let mut microseconds_since_previous_event = 0.0;

        while let Some(event) = self.track_iterator.next() {
            // `vtime` is in ticks.
            microseconds_since_previous_event +=
                (event.vtime as f64) / self.ticks_per_microsecond();

            match &event.event {
                Event::Midi(mm) => {
                    if mm.data.len() != 3 {
                        unimplemented!("better error handling for this error case");
                    }
                    return Some(DeltaEvent {
                        microseconds_since_previous_event,
                        event: RawMidiEvent::new([mm.data[0], mm.data[1], mm.data[2]]),
                    });
                }
                Event::Meta(MetaEvent {
                    command: MetaCommand::TempoSetting,
                    length: _,
                    data,
                }) => {
                    if data.len() != 3 {
                        unimplemented!("better error handling for this error case");
                    }
                    self.current_tempo_in_micro_seconds_per_beat =
                        data[2] as f64 + 255.0 * (data[1] as f64 + (255.0 * data[0] as f64));
                }
                Event::Meta(_) => {}
            }
        }
        return None;
    }
}
