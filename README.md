# rsynth

A crate for developing audio plugins and applications in Rust, with a focus on software synthesis.
Rsynth is well suited as a bootstrap for common audio plugin generators. 
It handles voices, voice-stealing, polyphony, etc. so the programmer's main focus can be DSP. 

Rsynth has the following components:

* An API abstraction layer
* Glue code for different API's (called back-ends). Currently supported are
  * [rust-vst](https://github.com/rust-dsp/rust-vst)
  * Jack
* Middleware components that you can put between your code and the abstraction lyer to provide 
  various functionalities:
  * polyphony
  * ...

[Documentation](https://resamplr.github.io/rsynth)

# Current State

rsynth is in its early stage of development and many changes are still breaking changes.
The team behind it is very small, so progress is slow.

# Roadmap

We try to focus on features that we are actually using ourselves.
This helps to ensure that the features that we provide, can actually be used in practice.
So if you want to use a particular feature that isn't there yet, feel free to open an issue (if 
there is none yet) and you can volunteer to test the feature before it is merged. 

Features that are likely to be realized:

- Make the event handling system extensible for other event types
- Add the notion of context
- Add a back-end for testing
- Add middleware to split the audio-buffer so that timed events are at sample `0`
- Add support for envelopes

Features that are likely going to be postponed for a long time, depending on the capacity of the
team and other issues (unless somebody joins to help with these)

- Support for LV2

In the long term, rsynth can be split into multiple crates for maximum reusability
and for license clarity (e.g. when one back-end mandates a different license).
We're currently keeping everything together because it's easier to coordinate breaking changes
over the various components in this way.

# Contributing

Contributions and suggestions are welcome.

In order to avoid pull requests from being broken by other changes, please open an issue or
have an issue assigned to you before you start working on something. 
In this way, we can coordinate development.
Issues labeled with "good first issue" should not conflict too much with other changes
that are in flight, but better check before you start working on one.

# Sponsorship

rsynth is helped by [Resamplr.com](https://resamplr.com/), a virtual instrument website.

# License 

MIT/BSD-3 License
