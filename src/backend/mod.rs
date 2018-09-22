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

pub trait OutputAudioChannelGroup<'a, 'b, C, T>: Sized
    where C: OutputAudioChannel<'a, T>,
{
    fn len(&self) -> usize;
    fn get_mut(&'b mut self, index: usize) -> Option<C>;
    fn split_at_mut(&'a mut self, index: usize) -> (Self, Self);
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

impl<'a, T: 'a> OutputAudioChannelGroup<'a, 'a, &'a mut [T], T> for Outputs<'a, T> {
    fn len(&self) -> usize {
        Outputs::len(self)
    }

    fn get_mut(&'a mut self, index: usize) -> Option<&'a mut [T]> {
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
    use jack::{Port, AudioIn, AudioOut, ProcessScope, Client, ClosureProcessHandler, ClientOptions, Control};

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

    impl<'ps> InputAudioChannel<'ps, f32> for &'ps AudioInWrapper<'ps> {
        fn slice(&'ps self) -> &'ps [f32]{
            self.audio_in.as_slice(&self.ps)
        }
    }

    struct AudioOutWrapper<'ps> {
        ps: &'ps ProcessScope,
        audio_out: &'ps mut Port<AudioOut>
    }

    impl<'g, 'ps> OutputAudioChannel<'ps, f32> for &'g mut AudioOutWrapper<'ps> {
        fn slice(&'ps mut self) -> &'ps mut [f32] {
            self.audio_out.as_mut_slice(&self.ps)
        }
    }

    struct JackInputAudioChannelGroup<'ps> {
        channels: &'ps [AudioInWrapper<'ps>]
    }

    impl<'ps> InputAudioChannelGroup<'ps, &'ps AudioInWrapper<'ps>, f32> for JackInputAudioChannelGroup<'ps>
    {
        fn len(&self) -> usize {
            self.channels.len()
        }

        fn get(&self, index: usize) -> Option<&'ps AudioInWrapper<'ps>> {
            self.channels.get(index)
        }

        fn split_at(&self, index: usize) -> (Self, Self) {
            let (first_channels, last_channels) = self.channels.split_at(index);
            let first = JackInputAudioChannelGroup {
                channels: first_channels
            };
            let last = JackInputAudioChannelGroup {
                channels: last_channels
            };
            (first, last)
        }
    }

    struct JackOutputAudioChannelGroup<'g, 'ps> where 'ps: 'g{
        channels: &'g mut [AudioOutWrapper<'ps>]
    }

    impl<'g, 'ps> OutputAudioChannelGroup<'ps, 'g, &'g mut AudioOutWrapper<'ps>, f32> for JackOutputAudioChannelGroup<'g, 'ps> where 'ps: 'g{
        fn len(&self) -> usize {
            self.channels.len()
        }

        fn get_mut(&'g mut self, index: usize) -> Option<&'g mut AudioOutWrapper<'ps>> {
            self.channels.get_mut(index)
        }

        fn split_at_mut(&'g mut self, index: usize) -> (Self, Self) {
            let (first_channels, last_channels) = self.channels.split_at_mut(index);
            let first = JackOutputAudioChannelGroup {
                channels: first_channels
            };
            let last = JackOutputAudioChannelGroup {
                channels: last_channels
            };
            (first, last)
        }
    }

    // Is in fact a test.
    fn test_synth() {
        let (client, _status) =
            Client::new("client", ClientOptions::NO_START_SERVER).unwrap();

        let mut out_port = client
            .register_port("out", AudioOut::default())
            .unwrap();
        let cback = move |_: &Client, ps: &ProcessScope| -> Control {
            let out_wrapper = AudioOutWrapper{
                ps: ps,
                audio_out: &mut out_port
            };
            let mut wrapper_group = vec![out_wrapper];
            let mut group = JackOutputAudioChannelGroup {
                channels: &mut wrapper_group
            };
            let channel = group.get_mut(0).unwrap();
            // TODO: Fix compile error when uncommenting the next line.
//            let _slice : &mut[f32] = channel.slice();
            Control::Continue
        };
        let _active_client = client
            .activate_async((), ClosureProcessHandler::new(cback))
            .unwrap();
    }
}