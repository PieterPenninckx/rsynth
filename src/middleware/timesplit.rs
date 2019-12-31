use crate::event::event_queue::{AlwaysInsertNewAfterOld, EventQueue};
use crate::event::Timed;

pub struct TimeChunk<'f, E, S> {
    pub event: Option<E>,
    pub inputs: &'f [&'f [S]],
    pub outputs: &'f mut [&'f mut [S]],
}

pub struct TimeChunkIterator<'f, 's, E, S> {
    splitter: &'s TimeSplitter<E>,
    remaining_input: &'f [&'f [S]],
    remaining_output: &'f mut [&'f mut [S]],
}

// TODO: Implement iterator for TimeChunkIterator

pub struct TimeSplitter<E> {
    queue: EventQueue<E>,
}

impl<E> TimeSplitter<E> {
    pub fn new(capacity: usize) -> Self {
        Self {
            queue: EventQueue::new(capacity),
        }
    }

    pub fn queue_event(&mut self, event: Timed<E>) -> Option<Timed<E>> {
        self.queue.queue_event(event, AlwaysInsertNewAfterOld)
    }

    pub fn chunk<'f, 's, S>(
        &'s self,
        inputs: &'f [&'f [S]],
        outptus: &'f mut [&'f mut [S]],
    ) -> TimeChunkIterator<'f, 's, E, S> {
        TimeChunkIterator {
            splitter: self,
            remaining_input: inputs,
            remaining_output: outptus,
        }
    }

    // TODO: implement something like "forget_before
}
