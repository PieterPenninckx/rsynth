use crate::event::event_queue::EventQueue;
use crate::event::Timed;
use asprim::AsPrim;
use num_traits::Float;

pub struct TimeChunk<'f, E, F> {
    pub event: Option<E>,
    pub inputs: &'f [&'f [F]],
    pub outputs: &'f mut [&'f mut [F]],
}

pub struct TimeChunkIterator<'f, 's, E, F> {
    splitter: &'s TimeSplitter<E>,
    remaining_input: &'f [&'f [F]],
    remaining_output: &'f mut [&'f mut [F]],
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
        self.queue.queue_event(event)
    }

    pub fn chunk<'f, 's, F>(
        &'s self,
        inputs: &'f [&'f [F]],
        outptus: &'f mut [&'f mut [F]],
    ) -> TimeChunkIterator<'f, 's, E, F>
    where
        F: Float + AsPrim,
    {
        TimeChunkIterator {
            splitter: self,
            remaining_input: inputs,
            remaining_output: outptus,
        }
    }

    // TODO: implement something like "forget_before
}
