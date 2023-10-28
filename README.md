# rsynth

A deprecated API abstraction for API's for audio plugins and applications.
You could use it to write real-time audio effects, software synthesizers, ... and target different platforms
(jack, offline processing, ...).

## This crate has been deprecated
This crate has been deprecated. See [this blog post](https://nckx.be/blog/rsynth-deprecation/) for more information.

### What should I do if I use this crate?
At the time of writing, here are some options
* Use [cpal](https://crates.io/crates/cpal) if you want to "just" play audio on various platforms.
* Use [nih-plug](https://github.com/robbert-vdh/nih-plug) if this is a good solution for you.
* Write the plugin as a “core” library (a Rust crate or module). This is anyhow something I'd recommend, also if you use [`nih-plug`](https://github.com/robbert-vdh/nih-plug), for instance. Per plugin standard that you want to support, create a separate crate that depends both on the “core” library and on an a library that is dedicated to that particular plugin standard (such as the [`lv2`](https://crates.io/crates/lv2) crate and the [`clack`](https://github.com/prokopyl/clack) crate (not (yet?) on crates.io). 

## Old documentation
### Feature matrix

We focus on API's that are typically used for audio effects and software synthesizers.
If you want to "just" play audio on various platforms, [cpal](https://crates.io/crates/cpal) may
be better suited for you.

| feature |  VST 2.4 via [`vst-rs`]      | Jack via [`jack`] | Offline audio rendering |
|---------|:------------------------------:|:-----------------:|:-----------------------:|
| Full duplex audio input and output |  ✓  |        ✓          |           ✓             |
| Midi input                         |  ✓  |        ✓          |           ✓             |
| Midi output                        | N/A |        ✓          |           ✘             |
| Sample accurate midi               | N/A |        ✓          |           ✓             |
| Multiple midi inputs and outputs   | N/A |        ✓          |           ✘             |
| Sampling frequency change          |  ✓  |        ✘          |          N/A            |
| Signal stopping the application    | N/A |        ✓          |           ✓             |
| Jack-specific events               | N/A |        ✘          |          N/A            |
| Basic meta-data                    |  ✓  |        ✓          |          N/A            |
| Access to the underlying host      |  ✓  |        ✓          |          N/A            |
| Parameter changes                  |  ✘  |        ✘          |           ✘             |
| GUI support                        |  ✘  |        ✘          |           ✘             |

#### Feature flags

Many features are behind feature flags: 
* `all`: all the features below
  * `backend-jack`: create standalone `jack` applications.
  * `backend-vst`: create VST 2.4 plugins.
  * `backend-combined-all`: all the "combined" backends for offline processing and testing. This always include in-memory dummy and testing backends.
    * `backend-combined-hound`: read and write `.wav` files with the `hound` crate
    * `backend-combined-wav-0-6`: read and write `.wav` files with the `wav` crate
    * `backend-combined-midly-0-5`: read and write `.mid` files with the `midly` crate 
  * `rsor-0-1`: add support for using the `rsor` crate for some methods (if you prefer `rsor` over `vecstorage`)

### Documentation

The API documentation can be found
* [on docs.rs](https://docs.rs/rsynth/) for the version that is distributed via crates.io.
* [on GitHub pages](https://pieterpenninckx.github.io/rsynth/rsynth) for the documentation of the master branch
* on your local machine after running `cargo rustdoc --features all`

### Philosophy and design
`rsynth` presents itself as a library, rather than a framework. 
Rather than trying to solve every problem (which is not feasible for the small team), 
`rsynth` is designed to be easy to combine with other crates for specific tasks, such as
* [`polyphony`](https://crates.io/crates/polyphony): the name says it all
* [`wmidi`](https://crates.io/crates/wmidi): encode and decode midi messages in real-time
* [`midi-consts`](https://crates.io/crates/midi-consts): constants for low-level handling of midi data
* [`rtrb`](crates.io/crates/rtrb), a realtime-safe single-producer single-consumer ring buffer that can be used to communicate between threads.

Background on the design can be found in the [design.md](design.md) document.

### Examples
There are full examples in 
[the examples folder in the source code](https://github.com/PieterPenninckx/rsynth/tree/master/examples).

## Current State
This crate has been deprecated. See [this blog post](https://nckx.be/blog/rsynth-deprecation/) for more information.

# License 

The source code of `rsynth` is licensed under the MIT/BSD-3 License.

Note that in order to use `rsynth` in combination with other crates (libraries), the combined work needs
to comply with the license of that crate as well. In particular, the following optional dependencies may require your attention:
* the `hound` crate (behind the `backend-combined-hound` feature) uses the Apache license, see [its readme](https://github.com/ruuda/hound#license) for more details
* the `wav` crate (behind the `backend-combined-wav` feature) uses the LGPL license

[`vst-rs`]: https://github.com/RustAudio/vst-rs
[`jack`]:https://crates.io/crates/jack
