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
    pub channel: u8
}

/// Return a default `NoteData` object, with full velocity and a note of middle C.
impl Default for NoteData {
    fn default() -> NoteData {
        NoteData { note: 60u8, velocity: 127u8, state: NoteState::Nil, channel: 0 }
    }
}

/// Builder methods for `NoteData`
impl NoteData {
    /// Return a new default `NoteData` structure
    pub fn new() -> Self {
        NoteData::default()
    }

    /// Modify the note and return a mutable reference to self
    pub fn note(&mut self, note: u8) -> &mut Self {
        self.note = note;
        self
    }

    /// Modify the velocity and return a mutable reference to self
    pub fn velocity(&mut self, velocity: u8) -> &mut Self {
        self.velocity = velocity;
        self
    }

    /// Modify the channel and return a mutable reference to self
    pub fn channel(&mut self, channel: u8) -> &mut Self {
        self.channel = channel;
        self
    }

    /// Modify the state and return a mutable reference to self
    pub fn state(&mut self, state: NoteState) -> &mut Self {
        self.state = state;
        self
    }

    /// Finalize all data and return a new `NoteData` structure
    pub fn finalize(self) -> Self {
        NoteData { note: self.note, velocity: self.velocity, channel: self.channel, state: self.state }
    }

    /// Convert note data obtained from the host into a `NoteData` structure.
    pub fn data(data: [u8; 3]) -> NoteData {
		let (state, channel) = NoteState::state_and_channel(data[0]);
		NoteData { 
			state: state, 
			note: data[1], 
			velocity: data[2],
			channel: channel }
	}
}

/// 11110000
const STATUS_MASK: u8 = 0xF0;
/// 00001111
const CHANNEL_MASK: u8 = 0x0F;

/// A more readable boolean for keeping track of a note's state
#[derive(PartialEq, Clone)]
pub enum NoteState {
    Nil,
    /// The note is off and should start `Releasing` a voice, if applicable
    Off,
    /// The note is on
    On
}

impl NoteState {
    pub fn state_and_channel(val: u8) -> (NoteState, u8) {
        let status = val & STATUS_MASK;
        let channel = val & CHANNEL_MASK;
        let status_enum = match status {
            0x80 => NoteState::Off,
            0x90 => NoteState::On,
            _ =>    NoteState::Nil
        };
        (status_enum, channel)
    }
}
