use super::{Envelope, EnvelopeIteratorItem};
use crate::event::Timed;

pub struct StairCaseEnvelopeIterator<'a, T>
where
    T: Copy,
{
    envelope: &'a StairCaseEnvelope<T>,
    index: usize,
    // Time to live
    ttl: usize,
    current_value: T,
}

impl<'a, T> StairCaseEnvelopeIterator<'a, T>
where
    T: Copy + 'a,
{
    fn new(envelope: &'a StairCaseEnvelope<T>) -> Self {
        Self {
            envelope,
            index: 0,
            ttl: envelope
                .event_buffer
                .first()
                .map(|x| x.time_in_frames as usize)
                .unwrap_or(usize::max_value()),
            current_value: envelope.initial_value,
        }
    }
}

impl<'a, T> Iterator for StairCaseEnvelopeIterator<'a, T>
where
    T: Copy + 'a,
{
    type Item = EnvelopeIteratorItem<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let has_updated;
        if self.ttl == 0 {
            has_updated = true;
            self.current_value = self.envelope.event_buffer[self.index].event;
            self.index += 1;
            self.ttl = if self.index < self.envelope.event_buffer.len() {
                (self.envelope.event_buffer[self.index].time_in_frames
                    - self.envelope.event_buffer[self.index - 1].time_in_frames)
                    as usize
            } else {
                usize::max_value()
            };
        } else {
            has_updated = false;
        }

        self.ttl -= 1;

        Some(EnvelopeIteratorItem {
            item: self.current_value,
            has_updated,
        })
    }
}

#[test]
fn staircase_envelope_iterator_next_called_with_empty_staircase_initial_value_returned() {
    let se = StairCaseEnvelope {
        initial_value: 4,
        event_buffer: vec![],
    };
    let mut iterator = se.iter();
    assert_eq!(iterator.next().map(|x| x.item), Some(4));
}

#[test]
fn staircase_envelope_iterator_next_called_with_nonempty_staircase_initial_value_returned() {
    let se = StairCaseEnvelope {
        initial_value: 1,
        event_buffer: vec![Timed::new(2, 4), Timed::new(3, 9), Timed::new(5, 25)],
    };
    let iterator = se.iter();
    assert_eq!(
        iterator.take(7).map(|x| x.item).collect::<Vec<_>>(),
        vec![1, 1, 4, 9, 9, 25, 25]
    );
}

#[derive(Clone)]
pub struct StairCaseEnvelope<T>
where
    T: Copy,
{
    initial_value: T,
    event_buffer: Vec<Timed<T>>,
}

impl<'a, T> Envelope<'a, T> for StairCaseEnvelope<T>
where
    T: Copy + 'a,
{
    type Iter = StairCaseEnvelopeIterator<'a, T>;
    type EventType = Timed<T>;

    fn iter(&'a self) -> Self::Iter {
        StairCaseEnvelopeIterator::new(self)
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
        // TODO: Use another (more readable) technique using the `position` method on an iterator.
        while read_index < self.event_buffer.len() {
            self.event_buffer[write_index] = self.event_buffer[read_index];
            if self.event_buffer[write_index].time_in_frames >= number_of_frames_to_forget {
                self.event_buffer[write_index].time_in_frames -= number_of_frames_to_forget;
                write_index += 1;
            } else {
                self.initial_value = self.event_buffer[write_index].event;
            }
            read_index += 1;
        }
        self.event_buffer.truncate(write_index);
    }
}

#[test]
fn staircaseenvelope_forget_past() {
    let mut se = StairCaseEnvelope {
        initial_value: 0,
        event_buffer: vec![
            Timed::new(4, 16),
            Timed::new(6, 36),
            Timed::new(7, 49),
            Timed::new(8, 64),
        ],
    };
    se.forget_past(7);
    assert_eq!(se.event_buffer, vec![Timed::new(0, 49), Timed::new(1, 64),]);
    assert_eq!(se.initial_value, 36);
}

#[test]
fn staircaseenvelope_forget_everything() {
    let mut se = StairCaseEnvelope {
        initial_value: 0,
        event_buffer: vec![
            Timed::new(4, 16),
            Timed::new(6, 36),
            Timed::new(7, 49),
            Timed::new(8, 64),
        ],
    };
    se.forget_past(9);
    assert_eq!(se.event_buffer, Vec::new());
    assert_eq!(se.initial_value, 64);
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
        initial_value: 0,
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
        initial_value: 0,
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
        initial_value: 0,
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
        initial_value: 0,
        event_buffer: initial_buffer.clone(),
    };
    se.event_buffer.reserve(1);

    // Act
    se.insert_event(Timed::new(6, 25));

    // Assert:
    assert_eq!(se.event_buffer, initial_buffer);
}
