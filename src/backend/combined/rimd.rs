use super::{MidiReader, MICROSECONDS_PER_SECOND};
use crate::backend::combined::MidiWriter;
use crate::event::{DeltaEvent, RawMidiEvent};
use rimd::{Event, MetaCommand, MetaEvent, MidiMessage, SMFBuilder, TrackEvent, SMF};

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
                    if let Some(raw_event) = RawMidiEvent::try_new(&mm.data) {
                        return Some(DeltaEvent {
                            microseconds_since_previous_event: microseconds_since_previous_event
                                as u64,
                            event: raw_event,
                        });
                    } else {
                        unimplemented!("better error handling for this error case");
                    }
                }
                Event::Meta(MetaEvent {
                    command: MetaCommand::TempoSetting,
                    data,
                    ..
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
        None
    }
}

pub struct RimdMidiWriter {
    writer: SMFBuilder,
    current_time_in_microseconds: u64,
    current_tempo_in_micro_seconds_per_beat: u32,
    ticks_per_beat: u16,
}

impl RimdMidiWriter {
    pub fn new(current_tempo_in_micro_seconds_per_beat: u32, ticks_per_beat: u16) -> Self {
        assert_eq!(ticks_per_beat & 0b10000000_00000000, 0);
        let mut writer = SMFBuilder::new();
        writer.add_track();
        writer.add_meta_abs(
            0,
            0,
            MetaEvent::tempo_setting(current_tempo_in_micro_seconds_per_beat),
        );
        Self {
            writer,
            current_time_in_microseconds: 0,
            current_tempo_in_micro_seconds_per_beat,
            ticks_per_beat,
        }
    }

    fn ticks_per_microsecond(&self) -> f64 {
        (self.ticks_per_beat as f64) / (self.current_tempo_in_micro_seconds_per_beat as f64)
    }

    pub fn get_smf(self) -> SMF {
        let Self {
            writer,
            ticks_per_beat,
            ..
        } = self;
        let mut result = writer.result();
        result.division = ticks_per_beat as i16;
        result
    }
}

impl MidiWriter for RimdMidiWriter {
    fn write_event(&mut self, event: DeltaEvent<RawMidiEvent>) {
        let DeltaEvent {
            microseconds_since_previous_event,
            event,
        } = event;
        self.current_time_in_microseconds += microseconds_since_previous_event;
        let current_time_in_ticks =
            self.current_time_in_microseconds as f64 / self.ticks_per_microsecond();
        self.writer.add_midi_abs(
            0,
            current_time_in_ticks as u64,
            MidiMessage::from_bytes(Vec::from(&event.data()[..])),
        );
    }
}
