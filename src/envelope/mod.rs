#![cfg_attr(not(feature = "stable"), feature(specialization))]

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EnvelopeIteratorItem<T> {
    pub item: T,
    pub has_updated: bool,
}

/// Defines the behaviour of an envelope.
/// An envelope allows to get an iterator.
/// The returned iterator allows to iterator over the frames, starting from
/// the current position, and for each frame, returns the envelope value at that frame.
// Note about the lifetime: ideally, we would use higher-kinded-types for this,
// but right now, that's not yet supported in Rust, so we do it this way.
pub trait Envelope<'a, T> {
    /// The type of the iterator.
    type Iter: Iterator<Item = EnvelopeIteratorItem<T>>;
    type EventType;
    /// Get the iterator.
    fn iter(&'a self) -> Self::Iter;
    fn insert_event(&mut self, event: Self::EventType);
    fn forget_past(&mut self, number_of_frames_to_forget: u32);
}

pub mod staircase_envelope;
