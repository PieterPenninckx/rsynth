use super::{Envelope, EnvelopeIteratorItem};
use crate::event::event_queue::{AlwaysRemoveOld, EventQueue};
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
                .event_queue
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

    // TODO: see if this cannot be moved to `EventQueue`.
    fn next(&mut self) -> Option<Self::Item> {
        let has_updated;
        if self.ttl == 0 {
            has_updated = true;
            self.current_value = self.envelope.event_queue[self.index].event;
            self.index += 1;
            self.ttl = if self.index < self.envelope.event_queue.len() {
                (self.envelope.event_queue[self.index].time_in_frames
                    - self.envelope.event_queue[self.index - 1].time_in_frames)
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

pub struct StairCaseEnvelope<T>
where
    T: Copy,
{
    initial_value: T,
    event_queue: EventQueue<T>,
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
        self.event_queue.queue_event(new_event, AlwaysRemoveOld);
    }

    fn forget_past(&mut self, number_of_frames_to_forget: u32) {
        if let Some(ref event) = self.event_queue.get_last_before(number_of_frames_to_forget) {
            self.initial_value = event.event;
        }
        self.event_queue.forget_before(number_of_frames_to_forget);
        self.event_queue.shift_time(number_of_frames_to_forget);
    }
}
