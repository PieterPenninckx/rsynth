//! Queue events.
use std::cmp::Ordering;
use std::collections::vec_deque::{Drain, VecDeque};
use std::iter::FusedIterator;
use std::ops::{Deref, Index, IndexMut, SubAssign};

/// A queue for timed events.
pub struct EventQueue<T, E> {
    queue: VecDeque<(T, E)>,
}

/// Determines what should happen when two events are queued with the same timing.
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

/// Trait that describes how "event collision" (queuing two events with the same timestamp) should happen.
pub trait HandleEventCollision<E> {
    fn decide_on_collision(&self, old_event: &E, new_event: &E) -> EventCollisionHandling;
}

/// Always queue the new newly queued event before the previously queued in case of collision (same timestamp).
pub struct AlwaysInsertNewBeforeOld;
impl<E> HandleEventCollision<E> for AlwaysInsertNewBeforeOld {
    #[inline(always)]
    fn decide_on_collision(&self, _old_event: &E, _new_event: &E) -> EventCollisionHandling {
        EventCollisionHandling::InsertNewBeforeOld
    }
}

/// Always queue the new newly queued event after the previously queued in case of collision (same timestamp).
pub struct AlwaysInsertNewAfterOld;
impl<E> HandleEventCollision<E> for AlwaysInsertNewAfterOld {
    #[inline(always)]
    fn decide_on_collision(&self, _old_event: &E, _new_event: &E) -> EventCollisionHandling {
        EventCollisionHandling::InsertNewAfterOld
    }
}

/// Always ignore the newly queued event in case of collision (there's already an event with that timestamp).
pub struct AlwaysIgnoreNew;
impl<E> HandleEventCollision<E> for AlwaysIgnoreNew {
    #[inline(always)]
    fn decide_on_collision(&self, _old_event: &E, _new_event: &E) -> EventCollisionHandling {
        EventCollisionHandling::IgnoreNew
    }
}

/// Always remove the previously queued event in case of collision (there's already an event with that timestamp).
pub struct AlwaysRemoveOld;
impl<E> HandleEventCollision<E> for AlwaysRemoveOld {
    #[inline(always)]
    fn decide_on_collision(&self, _old_event: &E, _new_event: &E) -> EventCollisionHandling {
        EventCollisionHandling::RemoveOld
    }
}

impl<T, E> Index<usize> for EventQueue<T, E> {
    type Output = (T, E);

    fn index(&self, index: usize) -> &Self::Output {
        &self.queue[index]
    }
}

impl<T, E> IndexMut<usize> for EventQueue<T, E> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.queue[index]
    }
}

impl<T, E> EventQueue<T, E> {
    /// Create a new `EventQueue` fom a vector of events.
    /// _Note_: this may violate the invariants of the `EventQueue`, so it's only available for testing.
    #[cfg(test)]
    pub fn from_vec(events: Vec<(T, E)>) -> Self {
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
    ///
    /// # Parameters
    /// (new_time, new_event): the new time, absolute time, to be added and the new event to be added
    pub fn queue_event<H>(
        &mut self,
        (new_time, new_event): (T, E),
        collision_decider: H,
    ) -> Option<(T, E)>
    where
        H: HandleEventCollision<E>,
        T: Ord,
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
            if new_time > self.queue[0].0 {
                result = self.queue.pop_front();
            } else {
                return Some((new_time, new_event));
            }
        } else {
            result = None;
        }
        // If we are at this point, we can assume that we can insert at least one more event.
        debug_assert!(self.queue.len() < self.queue.capacity());

        let mut insert_index = 0;
        for read_event in self.queue.iter_mut() {
            match read_event.0.cmp(&new_time) {
                Ordering::Less => {
                    insert_index += 1;
                }
                Ordering::Equal => {
                    match collision_decider.decide_on_collision(&read_event.1, &new_event) {
                        EventCollisionHandling::IgnoreNew => {
                            return Some((new_time, new_event));
                        }
                        EventCollisionHandling::InsertNewBeforeOld => {
                            break;
                        }
                        EventCollisionHandling::InsertNewAfterOld => {
                            insert_index += 1;
                        }
                        EventCollisionHandling::RemoveOld => {
                            std::mem::swap(&mut read_event.1, &mut new_event);
                            return Some((new_time, new_event));
                        }
                    }
                }
                Ordering::Greater => {
                    break;
                }
            }
        }
        self.queue.insert(insert_index, (new_time, new_event));

        result
    }

    /// Remove all events before, but not on, this threshold.
    ///
    /// # Note about usage in real-time context
    /// If `T` implements drop, the elements that are removed are dropped.
    /// This may cause memory de-allocation, which you want to avoid in
    /// the real-time part of your library.
    pub fn forget_before(&mut self, threshold: T)
    where
        T: Copy + Ord,
    {
        self.queue.retain(|x| x.0 >= threshold);
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
    pub fn shift_time(&mut self, new_zero_time: T)
    where
        T: Copy + SubAssign,
    {
        for event in self.queue.iter_mut() {
            event.0 -= new_zero_time;
        }
    }

    pub fn get_last_before(&self, time: T) -> Option<&(T, E)>
    where
        T: Ord,
    {
        if let Some(index) = self.queue.iter().rposition(|e| e.0 < time) {
            self.queue.get(index)
        } else {
            None
        }
    }

    /// Get the first event from the `EventQueue` if there is one and return `None` if the queue is empty.
    pub fn first(&self) -> Option<&(T, E)> {
        self.queue.get(0)
    }

    /// Create an iterator that drains all elements before but not on the given time.
    pub fn drain(&mut self, time: T) -> DrainingIter<T, E>
    where
        T: Ord,
    {
        if let Some(index) = self.queue.iter().rposition(|e| e.0 < time) {
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
    pub fn drain_all(&mut self) -> DrainingIter<T, E> {
        DrainingIter {
            inner: self.queue.drain(0..),
        }
    }
}

impl<E, T> Deref for EventQueue<T, E> {
    type Target = VecDeque<(T, E)>;

    fn deref(&self) -> &Self::Target {
        &self.queue
    }
}

#[test]
fn eventqueue_queue_event_new_event_ignored_when_already_full_and_new_event_comes_first() {
    let initial_buffer = vec![(4, 16), (6, 36), (7, 49)];
    let mut queue = EventQueue::from_vec(initial_buffer.clone());
    // Check our assumption:
    assert_eq!(queue.queue.capacity(), queue.queue.len());

    queue.queue_event((9, 3), AlwaysIgnoreNew);

    assert_eq!(queue.queue, initial_buffer);
}

#[test]
fn event_queue_queue_event_first_event_removed_when_already_full_and_new_event_after_first() {
    let initial_buffer = vec![(4, 16), (6, 36), (7, 49)];
    let mut queue = EventQueue::from_vec(initial_buffer.clone());
    // Check our assumption:
    assert_eq!(queue.queue.capacity(), queue.queue.len());

    queue.queue_event((5, 25), AlwaysInsertNewAfterOld);

    assert_eq!(queue.queue, vec![(5, 25), (6, 36), (7, 49),]);
}

#[test]
fn eventqueue_queue_event_new_event_inserted_at_correct_location() {
    let initial_buffer = vec![(4, 16), (6, 36), (7, 49)];
    let mut queue = EventQueue::from_vec(initial_buffer.clone());
    queue.queue.reserve(1);

    queue.queue_event((5, 25), AlwaysInsertNewAfterOld);

    assert_eq!(queue.queue, vec![(4, 16), (5, 25), (6, 36), (7, 49),]);
}

#[test]
fn eventqueue_queue_event_with_always_ignore_new_new_event_ignored_when_already_event_at_that_location(
) {
    let initial_buffer = vec![(4, 16), (6, 36), (7, 49)];
    let mut queue = EventQueue::from_vec(initial_buffer.clone());
    queue.queue.reserve(1);

    // Act
    queue.queue_event((6, 25), AlwaysIgnoreNew);

    // Assert:
    assert_eq!(queue.queue, initial_buffer);
}

#[test]
fn eventqueue_queue_event_with_always_ignore_old_old_event_ignored_when_already_event_at_that_location(
) {
    let initial_buffer = vec![(4, 16), (6, 36), (7, 49)];
    let expected_buffer = vec![(4, 16), (6, 25), (7, 49)];
    let mut queue = EventQueue::from_vec(initial_buffer.clone());
    queue.queue.reserve(1);

    // Act
    let result = queue.queue_event((6, 25), AlwaysRemoveOld);

    assert_eq!(result, Some((6, 36)));

    // Assert:
    assert_eq!(queue.queue, expected_buffer);
}

#[test]
fn eventqueue_queue_event_with_always_insert_new_after_old() {
    let initial_buffer = vec![(4, 16), (6, 36), (7, 49)];
    let expected_buffer = vec![(4, 16), (6, 36), (6, 25), (7, 49)];
    let mut queue = EventQueue::from_vec(initial_buffer.clone());
    queue.queue.reserve(1);

    // Act
    let result = queue.queue_event((6, 25), AlwaysInsertNewAfterOld);

    assert_eq!(result, None);

    // Assert:
    assert_eq!(queue.queue, expected_buffer);
}

#[test]
fn eventqueue_queue_event_with_always_insert_new_after_old_with_doubles() {
    let initial_buffer = vec![(6, 16), (6, 36), (7, 49)];
    let expected_buffer = vec![(6, 16), (6, 36), (6, 25), (7, 49)];
    let mut queue = EventQueue::from_vec(initial_buffer.clone());
    queue.queue.reserve(1);

    // Act
    let result = queue.queue_event((6, 25), AlwaysInsertNewAfterOld);

    assert_eq!(result, None);

    // Assert:
    assert_eq!(queue.queue, expected_buffer);
}

#[test]
fn eventqueue_queue_event_with_always_insert_new_before_old() {
    let initial_buffer = vec![(4, 16), (6, 36), (7, 49)];
    let expected_buffer = vec![(4, 16), (6, 25), (6, 36), (7, 49)];
    let mut queue = EventQueue::from_vec(initial_buffer.clone());
    queue.queue.reserve(1);

    // Act
    let result = queue.queue_event((6, 25), AlwaysInsertNewBeforeOld);

    assert_eq!(result, None);

    // Assert:
    assert_eq!(queue.queue, expected_buffer);
}

#[test]
fn eventqueue_forget_before() {
    let mut queue = EventQueue::from_vec({ vec![(4, 16), (6, 36), (7, 49), (8, 64)] });
    queue.forget_before(7);
    assert_eq!(queue.queue, vec![(7, 49), (8, 64),]);
}

#[test]
fn eventqueue_forget_everything() {
    let mut queue = EventQueue::from_vec({ vec![(4, 16), (6, 36), (7, 49), (8, 64)] });
    queue.forget_before(9);
    assert_eq!(queue.queue, Vec::new());
}

/// Draining iterator created by the [`EventQueue::drain`] method.
pub struct DrainingIter<'a, T, E> {
    inner: Drain<'a, (T, E)>,
}

impl<'a, T, E> Iterator for DrainingIter<'a, T, E> {
    type Item = (T, E);
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, T, E> DoubleEndedIterator for DrainingIter<'a, T, E> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

impl<'a, T, E> ExactSizeIterator for DrainingIter<'a, T, E> {}

impl<'a, T, E> FusedIterator for DrainingIter<'a, T, E> {}
