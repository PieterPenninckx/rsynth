# rsynth

An API abstraction for API's for audio plugins and applications.
Use it to write real-time audio effects, software synthesizers, ... and target different platforms
(vst, jack, ...).
It is currently most suitable for real-time or "streaming" audio processing.
E.g. you cannot use it to reverse audio in time.

## Feature matrix

We focus on API's that are typically used for audio effects and software synthesizers.
If you want to "just" play audio on various platforms, [cpal](https://crates.io/crates/cpal) may
be better suited for you.

Note: many of these features are behind feature flags.

| feature |  VST 2.4 via [`rust-vst`]      | Jack via [`jack`] | Offline audio rendering |
|---------|:------------------------------:|:-----------------:|:-----------------------:|
| Full duplex audio input and output |  ✓  |        ✓          |           ✓             |
| Midi input                         |  ✓  |        ✓          |           ✓             |
| Midi output                        | N/A |        ✓          |           ✘             |
| Sample accurate midi               | N/A |        ✓          |           ✓             |
| Multiple midi inputs and outputs   | N/A |        ✓          |           ✘             |
| Sampling frequency change          |  ✓  |        ✘          |          N/A            |
| Signal stopping the application    | N/A |        ✓          |          N/A            |
| Jack-specific events               | N/A |        ✘          |          N/A            |
| Basic meta-data                    |  ✓  |        ✓          |          N/A            |
| Access to the underlying host      |  ✓  |        ✓          |          N/A            |
| Parameter changes                  |  ✘  |        ✘          |           ✘             |
| GUI stuff                          |  ✘  |        ✘          |           ✘             |

## Documentation

The documentation can be found
* [on docs.rs](https://docs.rs/rsynth/) for the version that is distributed via crates.io.
* [on GitHub pages](https://pieterpenninckx.github.io/rsynth/rsynth) for the documentation of the master branch
* on your local machine after running `cargo rustdoc --features all`

## Examples
There are full examples in 
[the examples folder in the source code](https://github.com/PieterPenninckx/rsynth/tree/master/examples).


## Current State

`rsynth` is in its early stage of development and many changes are breaking changes.
The team behind it is very small, so progress is slow.

## Roadmap

We try to focus on features that we are actually using ourselves.
This helps to ensure that the features that we provide, can actually be used in practice.
So if you want to use a particular feature that isn't there yet, feel free to open an issue (if
needed) and you can volunteer to test the feature before it is merged.

In the long term, `rsynth` can be split into multiple crates for maximum re-usability
and for license clarity (e.g. when one back-end requires a different license).
We're currently keeping everything together because it's easier to coordinate breaking changes
over the various components in this way.

## Contributing

Contributions and suggestions are welcome!
See [CONTRIBUTING.md](CONTRIBUTING.md) for more details.

# Sponsorship

Alexander Lozada's contributions to rsynth are helped by [Resamplr.com](https://resamplr.com/), a virtual instrument website.

# License 

The source code of `rsynth` is licensed under the MIT/BSD-3 License.

Note: we plan to switch to MIT/Apache 2.0 in a future release.

Note that in order to use `rsynth` in combination with other crates (libraries), the combined work needs
to comply with the license of that crate as well. In particular, the following optional dependencies may require your attention:
* the `hound` crate (behind the `backend-combined-hound` feature) uses the Apache license, see [its readme](https://github.com/ruuda/hound#license) for more details
* the `wav` crate (behind the `backend-combined-wav` feature) uses the LGPL license

[`rust-vst`]: https://github.com/RustAudio/vst-rs
[`jack`]:https://crates.io/crates/jack
