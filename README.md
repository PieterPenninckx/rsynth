# rsynth

A crate for organizing synthesizers using [rust-vst](https://github.com/rust-dsp/rust-vst), inspired by JUCE's API.

rsynth provides a very lightweight `Synthesizer` structure, with many voices.  Once a `Voice` trait is implemented, it can be used easily from the `Synthesizer` manager.

[Documentation](https://resamplr.github.io/rsynth)

# Use Cases

rsynth is well suited as a bootstrap for common audio plugin generators.  rsynth will handle voices, voice-stealing, polyphony, etc. so the programmer's main focus can be DSP. 

rsynth is also split into multiple crates for maximum reusability.  Only include what you need to use!

# Roadmap

- [x] rsynth-core
  - [x] Voices
  - [X] Polyphony
  - [X] MIDI Processing
  - [X] Event Processing
  - [ ] Voice Stealing
  - [ ] Synthesizer
- [ ] rsynth-gui
  - [ ] GUI Integration
- [ ] rsynth-gen
  - [ ] Envelope Generators
  - [ ] Generic Oscillators
- [x] rsynth-dsp
  - [x] Equal Power Pan (to be moved)
  - [ ] TBD

# Current State

rsynth is not in a stable (or quite usable) state right now.  However, it may still be useful.  Contributions and suggestions are welcome.

# Sponsorship

rsynth is helped by [Resamplr.com](https://resamplr.com/), a virtual instrument website.

# License 

MIT/BSD-3 License
