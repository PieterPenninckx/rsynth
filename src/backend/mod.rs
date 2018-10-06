use std::ops::{Index, IndexMut};
use vst::buffer::{Inputs, Outputs};

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

#[cfg(feature="jack-backend")]
mod jack_backend {
    use jack::{Port, AudioIn, AudioOut, ProcessScope};
    #[cfg(test)]
    use jack::{Client, ClosureProcessHandler, ClientOptions, Control};

    use super::{
        InputAudioChannelGroup,
        OutputAudioChannelGroup
    };

    struct JackInputs<'ps, 'p> {
        process_scope: &'ps ProcessScope,
        audio_in_ports: &'p [Port<AudioIn>]
    }


    struct JackOutputs<'ps, 'p> {
        process_scope: &'ps ProcessScope,
        audio_out_ports: &'p mut [Port<AudioOut>]
    }

    impl<'ps, 'p> InputAudioChannelGroup<f32> for JackInputs<'ps, 'p>
    {
        fn number_of_channels(&self) -> usize {
            self.audio_in_ports.len()
        }

        fn channel_length(&self) -> usize {
            self.process_scope.n_frames() as usize
        }

        fn get(&self, index: usize) -> Option<&[f32]> {
            if let Some(channel) = self.audio_in_ports.get(index) {
                Some(channel.as_slice(&self.process_scope))
            } else {
                None
            }
        }

        fn split_at(self, index: usize) -> (Self, Self) {
            let (first_channels, last_channels) = self.audio_in_ports.split_at(index);
            let first = JackInputs {
                process_scope: self.process_scope,
                audio_in_ports: first_channels
            };
            let last = JackInputs {
                process_scope: self.process_scope,
                audio_in_ports: last_channels
            };
            (first, last)
        }
    }


    impl<'ps, 'p> OutputAudioChannelGroup<f32> for JackOutputs<'ps, 'p> {
        fn number_of_channels(&self) -> usize {
            self.audio_out_ports.len()
        }

        fn channel_length(&self) -> usize {
            self.process_scope.n_frames() as usize
        }

        fn get_mut(&mut self, index: usize) -> Option<&mut [f32]> {
            if let Some(channel) = self.audio_out_ports.get_mut(index) {
                Some(channel.as_mut_slice(&self.process_scope))
            } else {
                None
            }
        }

        fn split_at_mut(self, index: usize) -> (JackOutputs<'ps, 'p>, JackOutputs<'ps, 'p>) {
            let (first_channels, last_channels) = self.audio_out_ports.split_at_mut(index);
            let first = JackOutputs {
                process_scope: self.process_scope,
                audio_out_ports: first_channels
            };
            let last = JackOutputs {
                process_scope: self.process_scope,
                audio_out_ports: last_channels
            };
            (first, last)
        }
    }

    #[test]
    #[ignore]
    fn using_jack_client_compiles() {
        let (client, _status) =
            Client::new("client", ClientOptions::NO_START_SERVER).unwrap();

        let out_port = client
            .register_port("out", AudioOut::default())
            .unwrap();
        let mut out_ports = vec![out_port];
        let cback = move |_: &Client, ps: &ProcessScope| -> Control {
            let mut jack_outputs = JackOutputs{
                process_scope: ps,
                audio_out_ports: &mut out_ports
            };
            let _slice = jack_outputs.get_mut(0).unwrap();
            Control::Continue
        };
        let _active_client = client
            .activate_async((), ClosureProcessHandler::new(cback))
            .unwrap();
    }
}
