#![deprecated(since = "0.1.2", note = "Use the `event_queue` crate instead.")]
//! Queue events.
use super::Timed;
use crate::buffer::AudioBufferInOut;
use crate::event::EventHandler;
#[cfg(test)]
use crate::test_utilities::{DummyEventHandler, TestPlugin};
use crate::vecstorage::VecStorage;
use crate::ContextualAudioRenderer;
use std::cmp::Ordering;
use std::collections::vec_deque::{Drain, VecDeque};
use std::iter::FusedIterator;
use std::ops::{Deref, Index, IndexMut};
#[cfg_attr(test, allow(deprecated))]

/// A queue for timed events.
#[deprecated(since = "0.1.2", note = "Use the `event_queue` crate instead.")]
pub struct EventQueue<T> {
    queue: VecDeque<Timed<T>>,
}

/// Determines what should happen when two events are queued with the same timing.
#[deprecated(since = "0.1.2", note = "Use the `event_queue` crate instead.")]
pub enum EventCollisionHandling {
    /// Insert the newly queued event before the previously queued.
    InsertNewBeforeOld,
    /// Insert the newly queued event after the previously queued.
    InsertNewAfterOld,
    /// Ignore the newly queued event.
    IgnoreNew,
    /// Remove the previously queued event.
    RemoveOld,
}

/// Trait that describes how "event collision" (queing two events with the same timestamp) should happen.
#[deprecated(since = "0.1.2", note = "Use the `event_queue` crate instead.")]
pub trait HandleEventCollision<T> {
    fn decide_on_collision(&self, old_event: &T, new_event: &T) -> EventCollisionHandling;
}

/// Always queue the new newly queued event before the previously queued in case of collision (same timestamp).
#[deprecated(since = "0.1.2", note = "Use the `event_queue` crate instead.")]
pub struct AlwaysInsertNewBeforeOld;
impl<T> HandleEventCollision<T> for AlwaysInsertNewBeforeOld {
    #[inline(always)]
    fn decide_on_collision(&self, _old_event: &T, _new_event: &T) -> EventCollisionHandling {
        EventCollisionHandling::InsertNewBeforeOld
    }
}

/// Always queue the new newly queued event after the previously queued in case of collision (same timestamp).
#[deprecated(since = "0.1.2", note = "Use the `event_queue` crate instead.")]
pub struct AlwaysInsertNewAfterOld;
impl<T> HandleEventCollision<T> for AlwaysInsertNewAfterOld {
    #[inline(always)]
    fn decide_on_collision(&self, _old_event: &T, _new_event: &T) -> EventCollisionHandling {
        EventCollisionHandling::InsertNewAfterOld
    }
}

/// Always ignore the newly queued event in case of collision (there's already an event with that timestamp).
#[deprecated(since = "0.1.2", note = "Use the `event_queue` crate instead.")]
pub struct AlwaysIgnoreNew;
impl<T> HandleEventCollision<T> for AlwaysIgnoreNew {
    #[inline(always)]
    fn decide_on_collision(&self, _old_event: &T, _new_event: &T) -> EventCollisionHandling {
        EventCollisionHandling::IgnoreNew
    }
}

/// Always remove the previously queued event in case of collision (there's already an event with that timestamp).
#[deprecated(since = "0.1.2", note = "Use the `event_queue` crate instead.")]
pub struct AlwaysRemoveOld;
impl<T> HandleEventCollision<T> for AlwaysRemoveOld {
    #[inline(always)]
    fn decide_on_collision(&self, _old_event: &T, _new_event: &T) -> EventCollisionHandling {
        EventCollisionHandling::RemoveOld
    }
}

impl<T> Index<usize> for EventQueue<T> {
    type Output = Timed<T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.queue[index]
    }
}

impl<T> IndexMut<usize> for EventQueue<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.queue[index]
    }
}

impl<T> EventQueue<T> {
    /// Create a new `EventQueue` fom a vector of events.
    /// _Note_: this may violate the invariants of the `EventQueue`, so it's only available for testing.
    #[cfg(test)]
    pub fn from_vec(events: Vec<Timed<T>>) -> Self {
        Self {
            queue: events.into(),
        }
    }

    /// Create a new `EventQueue`.
    /// # Panics
    /// Panics if `capacity == 0`.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0);
        Self {
            queue: VecDeque::with_capacity(capacity),
        }
    }

    /// Queue a new event.
    /// When the buffer is full, an element may be removed from the queue to make some room.
    /// This element is returned.
    pub fn queue_event<H>(&mut self, new_event: Timed<T>, collision_decider: H) -> Option<Timed<T>>
    where
        H: HandleEventCollision<T>,
    {
        let mut new_event = new_event;
        let result;
        if self.queue.len() >= self.queue.capacity() {
            // Note: self.queue.capacity() > 0, so self.queue is not empty.
            // TODO: Log an error.
            // We remove the first event to come, in this way,
            // we are sure we are not skipping the "last" event,
            // because we assume that the state of the first event
            // is only temporarily, and the state of the last event
            // may remain forever. For this reason, it is safer to
            // remove the first event
            if new_event.time_in_frames > self.queue[0].time_in_frames {
                result = self.queue.pop_front();
            } else {
                return Some(new_event);
            }
        } else {
            result = None;
        }
        // If we are at this point, we can assume that we can insert at least one more event.
        debug_assert!(self.queue.len() < self.queue.capacity());

        let mut insert_index = 0;
        for read_event in self.queue.iter_mut() {
            match read_event.time_in_frames.cmp(&new_event.time_in_frames) {
                Ordering::Less => {
                    insert_index += 1;
                }
                Ordering::Equal => {
                    match collision_decider.decide_on_collision(&read_event.event, &new_event.event)
                    {
                        EventCollisionHandling::IgnoreNew => {
                            return Some(new_event);
                        }
                        EventCollisionHandling::InsertNewBeforeOld => {
                            break;
                        }
                        EventCollisionHandling::InsertNewAfterOld => {
                            insert_index += 1;
                        }
                        EventCollisionHandling::RemoveOld => {
                            std::mem::swap(&mut read_event.event, &mut new_event.event);
                            return Some(new_event);
                        }
                    }
                }
                Ordering::Greater => {
                    break;
                }
            }
        }
        self.queue.insert(insert_index, new_event);

        result
    }

    /// Remove all events before, but not on, this threshold.
    ///
    /// # Note about usage in real-time context
    /// If `T` implements drop, the elements that are removed are dropped.
    /// This may cause memory de-allocation, which you want to avoid in
    /// the real-time part of your library.
    pub fn forget_before(&mut self, threshold: u32)
    where
        T: Copy,
    {
        self.queue.retain(|x| x.time_in_frames >= threshold);
    }

    /// Remove all events from the queue.
    ///
    /// # Note about usage in real-time context
    /// If `T` implements drop, the elements that are removed are dropped.
    /// This may cause memory de-allocation, which you want to avoid in
    /// the real-time part of your library.
    pub fn clear(&mut self) {
        self.queue.clear()
    }

    /// Shift time forward by `new_zero_time` frames.
    ///
    /// # Panics
    /// Panics in debug mode when at least one event has a `time_in_frames`
    /// that is < `new_zero_time`.  
    pub fn shift_time(&mut self, new_zero_time: u32) {
        for event in self.queue.iter_mut() {
            event.time_in_frames -= new_zero_time;
        }
    }

    pub fn get_last_before(&self, time: u32) -> Option<&Timed<T>> {
        if let Some(index) = self.queue.iter().rposition(|e| e.time_in_frames < time) {
            self.queue.get(index)
        } else {
            None
        }
    }

    /// Get the first event from the `EventQueue` if there is one and return `None` if the queue is empty.
    pub fn first(&self) -> Option<&Timed<T>> {
        self.queue.get(0)
    }

    /// Go through the `EventQueue` and alternatingly handle events and render audio.
    ///
    /// # Note about using in a realtime context.
    /// There will be as many elements pushed to `input_storage` as there are
    /// input channels.
    /// There will be as many elements pushed to `output_storage` as there are
    /// output channels.
    #[deprecated(
        since = "0.1.2",
        note = "Use the `interleave` method on `AudioBufferInOut` instead."
    )]
    pub fn split<'in_storage, 'out_storage, 'in_channels, 's, 'chunk, S, R, C>(
        &mut self,
        input_storage: &'in_storage mut VecStorage<&'static [S]>,
        output_storage: &'out_storage mut VecStorage<&'static mut [S]>,
        buffer: &mut AudioBufferInOut<'in_channels, '_, '_, '_, S>,
        renderer: &mut R,
        context: &mut C,
    ) where
        S: Copy + 'static,
        R: ContextualAudioRenderer<S, C> + EventHandler<T>,
    {
        let buffer_length = buffer.number_of_frames();
        let mut last_event_time = 0;
        loop {
            if let Some(ref first) = self.queue.get(0) {
                if first.time_in_frames as usize >= buffer_length {
                    break;
                }
            } else {
                break;
            };
            let Timed {
                time_in_frames: event_time,
                event,
            } = self.queue.pop_front().expect("event queue is not empty");
            if event_time == last_event_time {
                renderer.handle_event(event);
                continue;
            }

            let mut input_guard = input_storage.vec_guard();
            let mut output_guard = output_storage.vec_guard();
            let mut sub_buffer = buffer.index_frames(
                (last_event_time as usize)..(event_time as usize),
                &mut input_guard,
                &mut output_guard,
            );
            renderer.render_buffer(&mut sub_buffer, context);
            renderer.handle_event(event);
            last_event_time = event_time;
        }
        if (last_event_time as usize) < buffer_length {
            let mut input_guard = input_storage.vec_guard();
            let mut output_guard = output_storage.vec_guard();
            let mut sub_buffer = buffer.index_frames(
                (last_event_time as usize)..buffer_length,
                &mut input_guard,
                &mut output_guard,
            );
            renderer.render_buffer(&mut sub_buffer, context);
        };
    }

    /// Create an iterator that drains all elements before but not on the given time.
    pub fn drain(&mut self, time: u32) -> DrainingIter<T> {
        if let Some(index) = self.queue.iter().rposition(|e| e.time_in_frames < time) {
            DrainingIter {
                inner: self.queue.drain(0..=index),
            }
        } else {
            DrainingIter {
                inner: self.queue.drain(0..0),
            }
        }
    }

    /// Create an iterator that drains all elements.
    pub fn drain_all(&mut self) -> DrainingIter<T> {
        DrainingIter {
            inner: self.queue.drain(0..),
        }
    }
}

impl<T> Deref for EventQueue<T> {
    type Target = VecDeque<Timed<T>>;

    fn deref(&self) -> &Self::Target {
        &self.queue
    }
}

#[test]
fn eventqueue_queue_event_new_event_ignored_when_already_full_and_new_event_comes_first() {
    let initial_buffer = vec![Timed::new(4, 16), Timed::new(6, 36), Timed::new(7, 49)];
    let mut queue = EventQueue::from_vec(initial_buffer.clone());
    // Check our assumption:
    assert_eq!(queue.queue.capacity(), queue.queue.len());

    // Act
    queue.queue_event(Timed::new(3, 9), AlwaysIgnoreNew);

    // Assert:
    assert_eq!(queue.queue, initial_buffer);
}

#[test]
fn event_queue_queue_event_first_event_removed_when_already_full_and_new_event_after_first() {
    let initial_buffer = vec![Timed::new(4, 16), Timed::new(6, 36), Timed::new(7, 49)];
    let mut queue = EventQueue::from_vec(initial_buffer.clone());
    // Check our assumption:
    assert_eq!(queue.queue.capacity(), queue.queue.len());

    // Act
    queue.queue_event(Timed::new(5, 25), AlwaysInsertNewAfterOld);

    // Assert:
    assert_eq!(
        queue.queue,
        vec![Timed::new(5, 25), Timed::new(6, 36), Timed::new(7, 49),]
    );
}

#[test]
fn eventqueue_queue_event_new_event_inserted_at_correct_location() {
    let initial_buffer = vec![Timed::new(4, 16), Timed::new(6, 36), Timed::new(7, 49)];
    let mut queue = EventQueue::from_vec(initial_buffer.clone());
    queue.queue.reserve(1);

    // Act
    queue.queue_event(Timed::new(5, 25), AlwaysInsertNewAfterOld);

    // Assert:
    assert_eq!(
        queue.queue,
        vec![
            Timed::new(4, 16),
            Timed::new(5, 25),
            Timed::new(6, 36),
            Timed::new(7, 49),
        ]
    );
}

#[test]
fn eventqueue_queue_event_with_always_ignore_new_new_event_ignored_when_already_event_at_that_location(
) {
    let initial_buffer = vec![Timed::new(4, 16), Timed::new(6, 36), Timed::new(7, 49)];
    let mut queue = EventQueue::from_vec(initial_buffer.clone());
    queue.queue.reserve(1);

    // Act
    queue.queue_event(Timed::new(6, 25), AlwaysIgnoreNew);

    // Assert:
    assert_eq!(queue.queue, initial_buffer);
}

#[test]
fn eventqueue_queue_event_with_always_ignore_old_old_event_ignored_when_already_event_at_that_location(
) {
    let initial_buffer = vec![Timed::new(4, 16), Timed::new(6, 36), Timed::new(7, 49)];
    let expected_buffer = vec![Timed::new(4, 16), Timed::new(6, 25), Timed::new(7, 49)];
    let mut queue = EventQueue::from_vec(initial_buffer.clone());
    queue.queue.reserve(1);

    // Act
    let result = queue.queue_event(Timed::new(6, 25), AlwaysRemoveOld);

    assert_eq!(result, Some(Timed::new(6, 36)));

    // Assert:
    assert_eq!(queue.queue, expected_buffer);
}

#[test]
fn eventqueue_queue_event_with_always_insert_new_after_old() {
    let initial_buffer = vec![Timed::new(4, 16), Timed::new(6, 36), Timed::new(7, 49)];
    let expected_buffer = vec![
        Timed::new(4, 16),
        Timed::new(6, 36),
        Timed::new(6, 25),
        Timed::new(7, 49),
    ];
    let mut queue = EventQueue::from_vec(initial_buffer.clone());
    queue.queue.reserve(1);

    // Act
    let result = queue.queue_event(Timed::new(6, 25), AlwaysInsertNewAfterOld);

    assert_eq!(result, None);

    // Assert:
    assert_eq!(queue.queue, expected_buffer);
}

#[test]
fn eventqueue_queue_event_with_always_insert_new_after_old_with_doubles() {
    let initial_buffer = vec![Timed::new(6, 16), Timed::new(6, 36), Timed::new(7, 49)];
    let expected_buffer = vec![
        Timed::new(6, 16),
        Timed::new(6, 36),
        Timed::new(6, 25),
        Timed::new(7, 49),
    ];
    let mut queue = EventQueue::from_vec(initial_buffer.clone());
    queue.queue.reserve(1);

    // Act
    let result = queue.queue_event(Timed::new(6, 25), AlwaysInsertNewAfterOld);

    assert_eq!(result, None);

    // Assert:
    assert_eq!(queue.queue, expected_buffer);
}

#[test]
fn eventqueue_queue_event_with_always_insert_new_before_old() {
    let initial_buffer = vec![Timed::new(4, 16), Timed::new(6, 36), Timed::new(7, 49)];
    let expected_buffer = vec![
        Timed::new(4, 16),
        Timed::new(6, 25),
        Timed::new(6, 36),
        Timed::new(7, 49),
    ];
    let mut queue = EventQueue::from_vec(initial_buffer.clone());
    queue.queue.reserve(1);

    // Act
    let result = queue.queue_event(Timed::new(6, 25), AlwaysInsertNewBeforeOld);

    assert_eq!(result, None);

    // Assert:
    assert_eq!(queue.queue, expected_buffer);
}

#[test]
fn eventqueue_forget_before() {
    let mut queue = EventQueue::from_vec({
        vec![
            Timed::new(4, 16),
            Timed::new(6, 36),
            Timed::new(7, 49),
            Timed::new(8, 64),
        ]
    });
    queue.forget_before(7);
    assert_eq!(queue.queue, vec![Timed::new(7, 49), Timed::new(8, 64),]);
}

#[test]
fn eventqueue_forget_everything() {
    let mut queue = EventQueue::from_vec({
        vec![
            Timed::new(4, 16),
            Timed::new(6, 36),
            Timed::new(7, 49),
            Timed::new(8, 64),
        ]
    });
    queue.forget_before(9);
    assert_eq!(queue.queue, Vec::new());
}

/// Draining iterator created by the [`EventQueue::drain`] method.
pub struct DrainingIter<'a, T> {
    inner: Drain<'a, Timed<T>>,
}

impl<'a, T> Iterator for DrainingIter<'a, T> {
    type Item = Timed<T>;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator for DrainingIter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

impl<'a, T> ExactSizeIterator for DrainingIter<'a, T> {}

impl<'a, T> FusedIterator for DrainingIter<'a, T> {}
