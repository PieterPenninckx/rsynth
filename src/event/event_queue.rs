use super::Timed;
use crate::buffer::AudioChunk;
use crate::event::{ContextualEventHandler, EventHandler};
use crate::test_utilities::TestPlugin;
use std::cmp::Ordering;
use std::ops::{Deref, Index, IndexMut};

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
}

impl<T> Deref for EventQueue<T> {
    type Target = [Timed<T>];

    fn deref(&self) -> &Self::Target {
        self.queue.as_slice()
    }
}

pub struct EventSlice<'e, E> {
    events: &'e [Timed<E>],
    offset: u32,
}

impl<'e, E> EventSlice<'e, E> {
    fn new(events: &'e [Timed<E>], offset: u32) -> Self {
        EventSlice { events, offset }
    }

    pub fn iter<'s>(&'s self) -> EventSliceIter<'s, 'e, E>
    where
        E: Copy,
    {
        EventSliceIter::new(self)
    }
}

pub struct EventSliceIter<'s, 'e, E>
where
    E: Copy,
{
    slice: &'s EventSlice<'e, E>,
    index: usize,
}

impl<'s, 'e, E> EventSliceIter<'s, 'e, E>
where
    E: Copy,
{
    pub fn new(slice: &'s EventSlice<'e, E>) -> Self {
        Self { slice, index: 0 }
    }
}

impl<'s, 'e, E> Iterator for EventSliceIter<'s, 'e, E>
where
    E: Copy,
{
    type Item = Timed<E>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(e) = self.slice.events.get(self.index) {
            self.index += 1;
            Some(Timed {
                event: e.event,
                time_in_frames: e.time_in_frames - self.slice.offset,
            })
        } else {
            None
        }
    }
}

#[test]
pub fn event_slice_iter_returns_none_for_empty_slice() {
    let events: Vec<Timed<()>> = Vec::new();
    let slice = EventSlice {
        offset: 1,
        events: &events,
    };
    let mut iter = slice.iter();
    assert_eq!(iter.next(), None);
}

#[test]
pub fn event_slice_iter_returns_shifted_element_for_slice_with_one_element() {
    let events = vec![Timed {
        time_in_frames: 1,
        event: 2,
    }];
    let slice = EventSlice {
        offset: 1,
        events: &events,
    };
    let mut iter = slice.iter();
    assert_eq!(
        iter.next(),
        Some(Timed {
            time_in_frames: 0,
            event: 2
        })
    );
    assert_eq!(iter.next(), None);
}

#[test]
pub fn event_slice_iter_returns_shifted_elements_for_slice_with_two_element() {
    let events = vec![
        Timed {
            time_in_frames: 3,
            event: 2,
        },
        Timed {
            time_in_frames: 6,
            event: 8,
        },
    ];
    let slice = EventSlice {
        offset: 2,
        events: &events,
    };
    let mut iter = slice.iter();
    assert_eq!(
        iter.next(),
        Some(Timed {
            time_in_frames: 1,
            event: 2
        })
    );
    assert_eq!(
        iter.next(),
        Some(Timed {
            time_in_frames: 4,
            event: 8
        })
    );
    assert_eq!(iter.next(), None);
}

pub struct TimeChunk<'e, 's, E, S> {
    pub events: EventSlice<'e, E>,
    pub inputs: &'s [&'s [S]],
    pub outputs: &'s mut [&'s mut [S]],
}

impl<'e, 's, E, S> TimeChunk<'e, 's, E, S>
where
    E: Copy,
{
    pub fn delegate_events<H>(&self, event_handler: &mut H)
    where
        H: EventHandler<Timed<E>>,
    {
        for event in self.events.iter() {
            event_handler.handle_event(event);
        }
    }

    pub fn delegate_events_contextually<H, C>(&self, event_handler: &mut H, context: &mut C)
    where
        H: ContextualEventHandler<Timed<E>, C>,
    {
        for event in self.events.iter() {
            event_handler.handle_event(event, context);
        }
    }
}

#[test]
fn delegate_events_with_one_event_works() {
    let inputs: Vec<&[f32]> = Vec::new();
    let events = vec![Timed {
        event: 4,
        time_in_frames: 5,
    }];
    let mut outputs: Vec<&mut [f32]> = Vec::new();
    let timechunk = TimeChunk {
        events: EventSlice {
            events: events.as_slice(),
            offset: 3,
        },
        inputs: inputs.as_slice(),
        outputs: outputs.as_mut_slice(),
    };
    let mut test_plugin = TestPlugin::<f32, _, _>::new(
        vec![AudioChunk::new(1)],
        vec![AudioChunk::new(1)],
        vec![vec![Timed {
            event: 4,
            time_in_frames: 2,
        }]],
        vec![Vec::new()],
        (),
    );
    timechunk.delegate_events(&mut test_plugin);
}

pub struct TimeChunkIterator<'q, 's, E, S> {
    remaining_events: &'q [E],
    remaining_input: &'s [&'s [S]],
    remaining_output: &'s mut [&'s mut [S]],
    offset: usize,
}

impl<'q, 's, E, S> TimeChunkIterator<'q, 's, E, S> {
    pub fn new(events: &'q [E], input: &'s [&'s [S]], output: &'s mut [&'s mut [S]]) -> Self {
        TimeChunkIterator {
            remaining_events: events,
            remaining_input: input,
            remaining_output: output,
            offset: 0,
        }
    }
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
