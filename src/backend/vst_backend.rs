use vst::buffer::{Inputs, Outputs};
use backend::InputAudioChannelGroup;
use backend::OutputAudioChannelGroup;

impl<'a, T:'a> InputAudioChannelGroup<T> for Inputs<'a, T> {
    fn number_of_channels(&self) -> usize {
        Inputs::len(self)
    }

    fn channel_length(&self) -> usize {
        if self.number_of_channels() > 0 {
            Inputs::get(self, 0).len()
        } else {
            0
        }
    }

    fn get(&self, index: usize) -> Option<&[T]> {
        if index < self.number_of_channels() {
            Some(&Inputs::get(self, index))
        } else {
            None
        }
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        // Takes `self` byvalue, not strictly necessary, only to be
        // consistent with the `split_at_mut`.
        Inputs::split_at(&self, index)
    }
}

impl<'a, T: 'a> OutputAudioChannelGroup<T> for Outputs<'a, T> {
    fn number_of_channels(&self) -> usize {
        Outputs::len(self)
    }

    fn channel_length(&self) -> usize {
        if self.number_of_channels() > 0 {
            Outputs::get(self, 0).len()
        } else {
            0
        }
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut [T]> {
        if index < self.number_of_channels() {
            Some(Outputs::get_mut(self, index))
        } else {
            None
        }
    }

    fn split_at_mut(self, index: usize) -> (Self, Self) {
        Outputs::split_at_mut(self, index)
    }
}