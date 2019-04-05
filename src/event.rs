pub struct RawMidiEvent<'a> {
    pub data: &'a [u8],
}

pub enum Event<T, U> {
    Timed { samples: u32, event: T },
    UnTimed(U),
}