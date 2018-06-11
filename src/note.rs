/// 11110000
const STATUS_MASK: u8 = 0xF0;
/// 00001111
const CHANNEL_MASK: u8 = 0x0F;
/// Default note is an A4
const DEFAULT_NOTE: u8 = 69u8;
/// Default velocity is full at 127
const DEFAULT_VELOCITY: u8 = 127u8;
/// The default channel is 0, which is usually what we're targeting.
const DEFAULT_CHANNEL: u8 = 0u8;

/// Contains all data needed to play a note
#[derive(Clone)]
pub struct NoteData {
    /// An integer from 0-127 defining what note to play based on the MIDI spec
    pub note: u8,
    /// An 8-bit unsigned value that can be used for modulating things such as amplitude
    pub velocity: u8,
    /// The On/Off state for a note
    pub state: NoteState,
    /// the intended channel
    pub channel: u8,
}

/// Return a default `NoteData` object, with full velocity and a note of middle C.
impl Default for NoteData {
    fn default() -> NoteData {
        NoteData {
            note: DEFAULT_NOTE,
            velocity: DEFAULT_VELOCITY,
            state: NoteState::Nil,
            channel: DEFAULT_CHANNEL,
        }
    }
}

/// This contains all data that can be constructed from a MIDI note signal.
impl NoteData {
    /// Convert note data obtained from the host into a `NoteData` structure.
    pub fn data(data: [u8; 3]) -> NoteData {
        let (state, channel) = NoteState::state_and_channel(data[0]);
        NoteData {
            state: state,
            note: data[1],
            velocity: data[2],
            channel: channel,
        }
    }
}

/// A more readable boolean for keeping track of a note's state
#[derive(PartialEq, Clone)]
pub enum NoteState {
    Nil,
    /// The note is off and should start `Releasing` a voice, if applicable
    Off,
    /// The note is on
    On,
}

impl NoteState {
    pub fn state_and_channel(val: u8) -> (NoteState, u8) {
        let status = val & STATUS_MASK;
        let channel = val & CHANNEL_MASK;
        let status_enum = match status {
            0x80 => NoteState::Off,
            0x90 => NoteState::On,
            _ => NoteState::Nil,
        };
        (status_enum, channel)
    }
}
