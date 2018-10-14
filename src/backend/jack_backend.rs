use std::slice;
use std::ops::{Index, IndexMut};
use jack::{Port, AudioIn, AudioOut, ProcessScope};
#[cfg(test)]
use jack::{Client, ClosureProcessHandler, ClientOptions, Control};

use super::{
    InputAudioChannelGroup,
    OutputAudioChannelGroup
};

#[derive(Clone, Copy)]
struct JackInputs<'ps, 'p> {
    process_scope: &'ps ProcessScope,
    audio_in_ports: &'p [Port<AudioIn>]
}

impl<'ps, 'p> Index<usize> for JackInputs<'ps, 'p> {
    type Output = [f32];
    fn index(&self, index: usize) -> &Self::Output {
        self.audio_in_ports[index].as_slice(&self.process_scope)
    }
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

struct JackOutputs<'ps, 'p> {
    process_scope: &'ps ProcessScope,
    audio_out_ports: &'p mut [Port<AudioOut>]
}

impl<'ps, 'p> Index<usize> for JackOutputs<'ps, 'p> {
    type Output = [f32];
    fn index(&self, index: usize) -> &Self::Output {
        // TODO: Add tests for this.
        let port = &self.audio_out_ports[index];
        assert_eq!(port.client_ptr(), self.process_scope.client_ptr());
        let buff = unsafe {
            slice::from_raw_parts(
                port.buffer(self.process_scope.n_frames()) as *const f32,
                self.process_scope.n_frames() as usize,
            )
        };
        buff
    }
}

impl<'ps, 'p> IndexMut<usize> for JackOutputs<'ps, 'p> {
    fn index_mut(&mut self, index: usize) -> &mut [f32] {
        self.audio_out_ports[index].as_mut_slice(&self.process_scope)
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
