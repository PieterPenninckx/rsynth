use super::Timed;
use crate::buffer::{AudioChunk, ChunkSplitter, ChunkSplitterMut};
use crate::event::{ContextualEventHandler, EventHandler};
use crate::test_utilities::TestPlugin;
use std::cmp::Ordering;
use std::mem;
use std::ops::{Deref, Index, IndexMut};
use vecstorage::{VecGuard, VecStorage};

pub struct EventQueue<T> {
    queue: Vec<Timed<T>>,
}

pub enum EventCollisionHandling {
    InsertNewBeforeOld,
    InsertNewAfterOld,
    IgnoreNew,
    RemoveOld,
}

pub trait HandleEventCollision<T> {
    fn decide_on_collision(&self, old_event: &T, new_event: &T) -> EventCollisionHandling;
}

pub struct AlwaysInsertNewBeforeOld;
impl<T> HandleEventCollision<T> for AlwaysInsertNewBeforeOld {
    #[inline(always)]
    fn decide_on_collision(&self, _old_event: &T, _new_event: &T) -> EventCollisionHandling {
        EventCollisionHandling::InsertNewBeforeOld
    }
}

pub struct AlwaysInsertNewAfterOld;
impl<T> HandleEventCollision<T> for AlwaysInsertNewAfterOld {
    #[inline(always)]
    fn decide_on_collision(&self, _old_event: &T, _new_event: &T) -> EventCollisionHandling {
        EventCollisionHandling::InsertNewAfterOld
    }
}

pub struct AlwaysIgnoreNew;
impl<T> HandleEventCollision<T> for AlwaysIgnoreNew {
    #[inline(always)]
    fn decide_on_collision(&self, _old_event: &T, _new_event: &T) -> EventCollisionHandling {
        EventCollisionHandling::IgnoreNew
    }
}

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
    #[cfg(test)]
    pub fn from_vec(events: Vec<Timed<T>>) -> Self {
        Self { queue: events }
    }

    pub fn new(capacity: usize) -> Self {
        Self {
            queue: Vec::with_capacity(capacity),
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
            // TODO: Log an error.
            // We remove the first event to come, in this way,
            // we are sure we are not skipping the "last" event,
            // because we assume that the state of the first event
            // is only temporarily, and the state of the last event
            // may remain forever. For this reason, it is safer to
            // remove the first event
            if new_event.time_in_frames > self.queue[0].time_in_frames {
                result = Some(self.queue.remove(0));
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

    pub fn first(&self) -> Option<&Timed<T>> {
        self.queue.get(0)
    }

    fn split<'storage, 's, 'chunk, S, F>(
        &self,
        input_storage: &'storage mut VecStorage<&'static [S]>,
        output_storage: &'storage mut VecStorage<&'static mut [S]>,
        inputs: &[&[S]],
        outputs: &'s mut [&'s mut [S]],
        mut function: F,
    ) where
        S: 'static,
        F: FnMut(&[&[S]], &mut [&mut [S]]),
    {
        // TODO: Make this a for-loop over the events
        loop {
            // TODO: use real start and end (instead of just 0)
            let input_guard = mid(input_storage, inputs, 0, 0);
            // TODO: use real start and end (instead of just 0)
            let mut output_guard = mid_mut(output_storage, outputs, 0, 0);
            // TODO: Also handle the event(s)
            function(&input_guard, &mut output_guard);
        }
    }
}

impl<T> Deref for EventQueue<T> {
    type Target = [Timed<T>];

    fn deref(&self) -> &Self::Target {
        self.queue.as_slice()
    }
}

// TODO: Move to a better place in the module hierarchy.
pub fn mid<'storage, 'chunk, 's, S>(
    storage: &'storage mut VecStorage<&'static [S]>,
    chunk: &'chunk [&'s [S]],
    start: usize,
    end: usize,
) -> VecGuard<'storage, &'static [S], &'chunk [S]> {
    let mut remaining_chunk = chunk;
    let mut guard = storage.vec_guard();
    let mut len = remaining_chunk.len();
    while len > 0 {
        let (first_channel, other_channels) = remaining_chunk.split_at(1);
        let channel = &(first_channel[0]);
        let (first, _) = channel.split_at(end);
        let (_, middle) = first.split_at(start);
        guard.push(middle);
        remaining_chunk = other_channels;
        len = remaining_chunk.len();
    }
    guard
}

// TODO: Move to a better place in the module hierarchy.
pub fn mid_mut<'storage, 'chunk, 's, S>(
    storage: &'storage mut VecStorage<&'static mut [S]>,
    chunk: &'chunk mut [&'s mut [S]],
    start: usize,
    end: usize,
) -> VecGuard<'storage, &'static mut [S], &'chunk mut [S]> {
    let mut remaining_chunk = chunk;
    let mut guard = storage.vec_guard();
    let mut len = remaining_chunk.len();
    while len > 0 {
        let (first_channel, other_channels) = remaining_chunk.split_at_mut(1);
        let channel = &mut (first_channel[0]);
        let (first, _) = channel.split_at_mut(end);
        let (_, middle) = first.split_at_mut(start);
        guard.push(middle);
        remaining_chunk = other_channels;
        len = remaining_chunk.len();
    }
    guard
}

#[test]
fn eventqueue_queue_event_new_event_ignored_when_already_full_and_new_event_comes_first() {
    let initial_buffer = vec![
        Timed::new(4, 16),
        Timed::new(6, 36),
        Timed::new(7, 49),
        Timed::new(8, 64),
    ];
    let mut queue = EventQueue {
        queue: initial_buffer.clone(),
    };
    // Check our assumption:
    assert_eq!(queue.queue.capacity(), queue.queue.len());

    // Act
    queue.queue_event(Timed::new(3, 9), AlwaysIgnoreNew);

    // Assert:
    assert_eq!(queue.queue, initial_buffer);
}

#[test]
fn event_queue_queue_event_first_event_removed_when_already_full_and_new_event_after_first() {
    let initial_buffer = vec![
        Timed::new(4, 16),
        Timed::new(6, 36),
        Timed::new(7, 49),
        Timed::new(8, 64),
    ];
    let mut queue = EventQueue {
        queue: initial_buffer.clone(),
    };
    // Check our assumption:
    assert_eq!(queue.queue.capacity(), queue.queue.len());

    // Act
    queue.queue_event(Timed::new(5, 25), AlwaysInsertNewAfterOld);

    // Assert:
    assert_eq!(
        queue.queue,
        vec![
            Timed::new(5, 25),
            Timed::new(6, 36),
            Timed::new(7, 49),
            Timed::new(8, 64),
        ]
    );
}

#[test]
fn eventqueue_queue_event_new_event_inserted_at_correct_location() {
    let initial_buffer = vec![Timed::new(4, 16), Timed::new(6, 36), Timed::new(7, 49)];
    let mut queue = EventQueue {
        queue: initial_buffer.clone(),
    };
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
    let mut queue = EventQueue {
        queue: initial_buffer.clone(),
    };
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
    let mut queue = EventQueue {
        queue: initial_buffer.clone(),
    };
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
    let mut queue = EventQueue {
        queue: initial_buffer.clone(),
    };
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
    let mut queue = EventQueue {
        queue: initial_buffer.clone(),
    };
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
    let mut queue = EventQueue {
        queue: initial_buffer.clone(),
    };
    queue.queue.reserve(1);

    // Act
    let result = queue.queue_event(Timed::new(6, 25), AlwaysInsertNewBeforeOld);

    assert_eq!(result, None);

    // Assert:
    assert_eq!(queue.queue, expected_buffer);
}

#[test]
fn eventqueue_forget_before() {
    let mut queue = EventQueue {
        queue: vec![
            Timed::new(4, 16),
            Timed::new(6, 36),
            Timed::new(7, 49),
            Timed::new(8, 64),
        ],
    };
    queue.forget_before(7);
    assert_eq!(queue.queue, vec![Timed::new(7, 49), Timed::new(8, 64),]);
}

#[test]
fn eventqueue_forget_everything() {
    let mut queue = EventQueue {
        queue: vec![
            Timed::new(4, 16),
            Timed::new(6, 36),
            Timed::new(7, 49),
            Timed::new(8, 64),
        ],
    };
    queue.forget_before(9);
    assert_eq!(queue.queue, Vec::new());
}
