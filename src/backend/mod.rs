use vst::buffer::{Inputs, Outputs};

pub trait InputAudioChannel<'a, T> {
    fn slice(&'a self) -> &'a [T];
}

pub trait OutputAudioChannel<'a, T> {
    fn slice(&'a mut self) -> &'a mut[T];
}

impl<'a, T> InputAudioChannel<'a, T> for &'a [T] {
    fn slice(&'a self) -> &'a [T] {
        self
    }
}

impl<'a, T> OutputAudioChannel<'a, T> for &'a mut[T] {
    fn slice(&'a mut self) -> &'a mut [T] {
        self
    }
}

pub trait InputAudioChannelGroup<'a, C, T>: Sized
    where C: InputAudioChannel<'a, T>,
{
    fn len(&self) -> usize;
    fn get(&self, index: usize) -> Option<C>;
    fn split_at(&self, index: usize) -> (Self, Self);
}

pub trait OutputAudioChannelGroup<'a, C, T>: Sized
    where C: OutputAudioChannel<'a, T>,
{
    fn len(&self) -> usize;
    fn get_mut(&mut self, index: usize) -> Option<C>;
    fn split_at_mut(&mut self, index: usize) -> (Self, Self);
}

impl<'a, T:'a> InputAudioChannelGroup<'a, &'a [T], T> for Inputs<'a, T> {
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

impl<'a, T: 'a> OutputAudioChannelGroup<'a, &'a mut [T], T> for Outputs<'a, T> {
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
mod jack_backend {
    use jack::{Port, AudioIn, AudioOut, ProcessScope};

    use super::{
        InputAudioChannel,
        OutputAudioChannel,
        InputAudioChannelGroup,
        OutputAudioChannelGroup
    };

    struct AudioInWrapper<'ps> {
        ps: &'ps ProcessScope,
        audio_in: Port<AudioIn>
    }

    impl<'ps> InputAudioChannel<'ps, f32> for AudioInWrapper<'ps> {
        fn slice(&'ps self) -> &'ps [f32]{
            self.audio_in.as_slice(&self.ps)
        }
    }

    struct AudioOutWrapper<'ps> {
        ps: &'ps ProcessScope,
        audio_out: Port<AudioOut>
    }

    impl<'ps> OutputAudioChannel<'ps, f32> for AudioOutWrapper<'ps> {
        fn slice(&'ps mut self) -> &'ps mut [f32] {
            self.audio_out.as_mut_slice(&self.ps)
        }
    }

    struct JackInputAudioChannelGroup{
        channels: Vec<AudioIn>
    }


}