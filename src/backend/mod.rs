use std::ops::{Index, IndexMut};
mod vst_backend;
#[cfg(feature="jack-backend")]
mod jack_backend;

pub trait InputAudioChannelGroup<T> : Sized + Index<usize, Output=[T]>
{
    fn number_of_channels(&self) -> usize;
    fn channel_length(&self) -> usize;
    fn get(&self, index: usize) -> Option<&[T]>;
    fn split_at(self, channel_index: usize) -> (Self, Self);
}

pub trait OutputAudioChannelGroup<T>: Sized + IndexMut<usize, Output=[T]>
{
    fn number_of_channels(&self) -> usize;
    fn channel_length(&self) -> usize;
    fn get_mut(&mut self, index: usize) -> Option<&mut [T]>;
    fn split_at_mut(self, channel_index: usize) -> (Self, Self);
}
