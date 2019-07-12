use crate::context::TransparentContext;
use crate::event::{EventHandler, Timed};
use std::mem;

pub trait EnvelopeIterator: Iterator {
    fn has_uptated(&self) -> bool;
}

/// Defines the behaviour of an envelope.
/// An envelope allows to get an iterator.
/// The returned iterator allows to iterator over the frames, starting from
/// the current position, and for each frame, returns the envelope value at that frame.
pub trait Envelope<'a, T>: Clone {
    /// The type of the iterator.
    type Iter: EnvelopeIterator<Item = T>;
    type EventType;
    /// Get the iterator.
    fn iter(&'a self) -> Self::Iter;
    fn insert_event(&mut self, event: Self::EventType);
    fn forget_past(&mut self, number_of_frames_to_forget: u32);
}

pub struct StairCaseEnvelopeIterator<T> {
    todo: T,
}

impl<T> Iterator for StairCaseEnvelopeIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}

impl<T> EnvelopeIterator for StairCaseEnvelopeIterator<T> {
    fn has_uptated(&self) -> bool {
        unimplemented!()
    }
}

#[derive(Clone)]
pub struct StairCaseEnvelope<T>
where
    T: Copy,
{
    event_buffer: Vec<Timed<T>>,
}

impl<'a, T> Envelope<'a, T> for StairCaseEnvelope<T>
where
    T: Copy,
{
    type Iter = StairCaseEnvelopeIterator<T>;
    type EventType = Timed<T>;

    fn iter(&'a self) -> Self::Iter {
        unimplemented!();
    }

    fn insert_event(&mut self, new_event: Timed<T>) {
        if self.event_buffer.len() >= self.event_buffer.capacity() {
            // TODO: Log an error.
            // We remove the first event to come, in this way,
            // we are sure we are not skipping the "last" event,
            // because we assume that the state of the first event
            // is only temporarily, and the state of the last event
            // may remain forever. For this reason, it is safer to
            // remove the first event
            if new_event.time_in_frames > self.event_buffer[0].time_in_frames {
                self.event_buffer.remove(0);
            } else {
                return;
            }
        }
        // If we are at this point, we can assume that we can insert at least one more event.
        debug_assert!(self.event_buffer.len() < self.event_buffer.capacity());

        let mut insert_index = 0;
        for read_event in self.event_buffer.iter() {
            if read_event.time_in_frames < new_event.time_in_frames {
                insert_index += 1;
            } else {
                if read_event.time_in_frames == new_event.time_in_frames {
                    // Two events at the same time.
                    // This should not happen, we are ignoring this event.
                    // TODO: Log a warning.
                    return;
                }
                break;
            }
        }
        self.event_buffer.insert(insert_index, new_event);
    }

    fn forget_past(&mut self, number_of_frames_to_forget: u32) {
        let mut write_index = 0;
        let mut read_index = 0;
        while read_index < self.event_buffer.len() {
            self.event_buffer[write_index] = self.event_buffer[read_index];
            if self.event_buffer[write_index].time_in_frames >= number_of_frames_to_forget {
                self.event_buffer[write_index].time_in_frames -= number_of_frames_to_forget;
                write_index += 1;
            }
            read_index += 1;
        }
        self.event_buffer.truncate(write_index);
    }
}

#[test]
fn staircaseenvelope_forget_past() {
    let mut se = StairCaseEnvelope {
        event_buffer: vec![
            Timed::new(4, 16),
            Timed::new(6, 36),
            Timed::new(7, 49),
            Timed::new(8, 64),
        ],
    };
    se.forget_past(7);
    assert_eq!(se.event_buffer, vec![Timed::new(0, 49), Timed::new(1, 64),]);
}

#[test]
fn staircaseenvelope_forget_everything() {
    let mut se = StairCaseEnvelope {
        event_buffer: vec![
            Timed::new(4, 16),
            Timed::new(6, 36),
            Timed::new(7, 49),
            Timed::new(8, 64),
        ],
    };
    se.forget_past(9);
    assert_eq!(se.event_buffer, Vec::new());
}

#[test]
fn staircaseenvelope_insert_event_new_event_ignored_when_already_full_and_new_event_comes_first() {
    let initial_buffer = vec![
        Timed::new(4, 16),
        Timed::new(6, 36),
        Timed::new(7, 49),
        Timed::new(8, 64),
    ];
    let mut se = StairCaseEnvelope {
        event_buffer: initial_buffer.clone(),
    };
    // Check our assumption:
    assert_eq!(se.event_buffer.capacity(), se.event_buffer.len());

    // Act
    se.insert_event(Timed::new(3, 9));

    // Assert:
    assert_eq!(se.event_buffer, initial_buffer);
}

#[test]
fn staircaseenvelope_insert_event_first_event_removed_when_already_full_and_new_event_after_first()
{
    let initial_buffer = vec![
        Timed::new(4, 16),
        Timed::new(6, 36),
        Timed::new(7, 49),
        Timed::new(8, 64),
    ];
    let mut se = StairCaseEnvelope {
        event_buffer: initial_buffer.clone(),
    };
    // Check our assumption:
    assert_eq!(se.event_buffer.capacity(), se.event_buffer.len());

    // Act
    se.insert_event(Timed::new(5, 25));

    // Assert:
    assert_eq!(
        se.event_buffer,
        vec![
            Timed::new(5, 25),
            Timed::new(6, 36),
            Timed::new(7, 49),
            Timed::new(8, 64),
        ]
    );
}

#[test]
fn staircaseenvelope_insert_event_new_event_inserted_at_correct_location() {
    let initial_buffer = vec![Timed::new(4, 16), Timed::new(6, 36), Timed::new(7, 49)];
    let mut se = StairCaseEnvelope {
        event_buffer: initial_buffer.clone(),
    };
    se.event_buffer.reserve(1);

    // Act
    se.insert_event(Timed::new(5, 25));

    // Assert:
    assert_eq!(
        se.event_buffer,
        vec![
            Timed::new(4, 16),
            Timed::new(5, 25),
            Timed::new(6, 36),
            Timed::new(7, 49),
        ]
    );
}

#[test]
fn staircaseenvelope_insert_event_new_event_ignored_when_already_event_at_that_location() {
    let initial_buffer = vec![Timed::new(4, 16), Timed::new(6, 36), Timed::new(7, 49)];
    let mut se = StairCaseEnvelope {
        event_buffer: initial_buffer.clone(),
    };
    se.event_buffer.reserve(1);

    // Act
    se.insert_event(Timed::new(6, 25));

    // Assert:
    assert_eq!(se.event_buffer, initial_buffer);
}

pub trait EnvelopeContext {
    type Marker;
    type Data;
    fn data(&mut self) -> &mut Self::Data;
}

pub trait WithEnvelope<'a, T>
where
    T: EnvelopeContext + 'a,
{
    fn envelope(&'a mut self, marker: T::Marker) -> &'a mut T::Data;
}

impl<'a, T, U> WithEnvelope<'a, T> for U
where
    U: TransparentContext<T>,
    T: EnvelopeContext + 'a,
{
    fn envelope(&'a mut self, marker: T::Marker) -> &'a mut T::Data {
        self.get().data()
    }
}

mod aftertouch;
