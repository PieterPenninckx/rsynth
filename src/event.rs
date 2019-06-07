#[cfg(feature = "stable")]
use crate::dev_utilities::compatibility::*;
#[cfg(feature = "stable")]
use syllogism::{Distinction, Specialize};
#[cfg(feature = "stable")]
use syllogism_macro::impl_specialization;

/// The trait that plugins should implement in order to handle the given type of events.
pub trait EventHandler<E, C> {
    fn handle_event(&mut self, event: E, context: &mut C);
}

#[derive(Clone, Copy)]
pub struct SysExEvent<'a> {
    data: &'a [u8],
}

impl<'a> SysExEvent<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
}

/// A raw midi event.
/// Use this when you need to be able to clone the event.
#[derive(Clone, Copy)]
pub struct RawMidiEvent {
    data: [u8; 3],
}

impl RawMidiEvent {
    pub fn new(data: [u8; 3]) -> Self {
        Self { data }
    }
    pub fn data(&self) -> &[u8; 3] {
        &self.data
    }
}

pub struct Timed<E> {
    pub time_in_frames: u32,
    pub event: E,
}

impl<E> Clone for Timed<E>
where
    E: Clone,
{
    fn clone(&self) -> Self {
        Timed {
            time_in_frames: self.time_in_frames,
            event: self.event.clone(),
        }
    }
}

impl<E> Copy for Timed<E> where E: Copy {}

#[cfg(feature = "stable")]
impl<E, T> Specialize<Timed<T>> for Timed<E>
where
    E: Specialize<T>,
{
    fn specialize(self) -> Distinction<Timed<T>, Self> {
        let Timed {
            time_in_frames,
            event,
        } = self;
        match event.specialize() {
            Distinction::Generic(g) => Distinction::Generic(Timed {
                time_in_frames,
                event: g,
            }),
            Distinction::Special(s) => Distinction::Special(Timed {
                time_in_frames,
                event: s,
            }),
        }
    }
}

#[cfg(feature = "stable")]
impl_specialization!(
    trait NotInCrateRsynth;
    macro macro_for_rsynth;

    type SysExEvent<'a>;
    type RawMidiEvent;
    type Timed<E>;
);
