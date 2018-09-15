use vst::buffer::{Inputs, Outputs};
#[cfg(feature="jack-backend")]
use jack::AudioIn;

pub trait InputAudioChannel<T> {
    fn slice(&self) -> &[T];
}

pub trait OutputAudioChannel<T> {
    fn slice(&mut self) -> &mut[T];
}

impl<'a, T> InputAudioChannel<T> for &'a [T] {
    fn slice(& self) -> & [T] {
        self
    }
}


impl<'a, T> OutputAudioChannel<T> for &'a mut[T] {
    fn slice(& mut self) -> & mut [T] {
        self
    }
}

pub trait InputAudioChannelGroup<C, T>: Sized
    where C: InputAudioChannel<T>,
{
    fn len(&self) -> usize;
    fn get(&self, index: usize) -> Option<C>;
    fn split_at(&self, index: usize) -> (Self, Self);
}

pub trait OutputAudioChannelGroup<C, T>: Sized
    where C: OutputAudioChannel<T>,
{
    fn len(&self) -> usize;
    fn get_mut(&mut self, index: usize) -> Option<C>;
    fn split_at_mut(&mut self, index: usize) -> (Self, Self);
}

impl<'a, T:'a> InputAudioChannelGroup<&'a [T], T> for Inputs<'a, T> {
    fn len(&self) -> usize {
        Inputs::len(self)
    }

    fn get(&self, index: usize) -> Option<&'a [T]> {
        if index < self.len() {
            Some(&Inputs::get(self, index))
        } else {
            None
        }
    }

    fn split_at(&self, index: usize) -> (Self, Self) {
        Inputs::split_at(self, index)
    }
}

impl<'a, T: 'a> OutputAudioChannelGroup<&'a mut [T], T> for Outputs<'a, T> {
    fn len(&self) -> usize {
        Outputs::len(self)
    }

    fn get_mut(&mut self, index: usize) -> Option<&'a mut [T]> {
        if index < self.len() {
            Some(Outputs::get_mut(self, index))
        } else {
            None
        }
    }

    fn split_at_mut(&mut self, index: usize) -> (Self, Self) {
        Outputs::split_at_mut(self, index)
    }
}

#[cfg(feature="jack-backend")]
struct JackInputAudioChannelGroup{
    channels: Vec<AudioIn>
}