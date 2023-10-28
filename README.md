# rsynth

A deprecated API abstraction for API's for audio plugins and applications.
You could use it to write real-time audio effects, software synthesizers, ... and target different platforms
(jack, offline processing, ...).

## This crate has been deprecated
This crate has been deprecated.

### What should I do if I use this crate?
Here are some options
* Use [cpal](https://crates.io/crates/cpal) if you want to "just" play audio on various platforms.
* Use [nih-plug](https://github.com/robbert-vdh/nih-plug) if this is a good solution for you.
* Write the plugin as a “core” library (a Rust crate or module). This is anyhow something I'd recommend, also if you use [`nih-plug`](https://github.com/robbert-vdh/nih-plug), for instance. Per plugin standard that you want to support, create a separate crate that depends both on the “core” library and on an a library that is dedicated to that particular plugin standard (such as the [`lv2`](https://crates.io/crates/lv2) crate and the [`clack`](https://github.com/prokopyl/clack) crate (not (yet?) on crates.io). 

## License 

The source code of `rsynth` is licensed under the MIT/BSD-3 License.

Note that in order to use `rsynth` in combination with other crates (libraries), the combined work needs
to comply with the license of that crate as well. In particular, the following optional dependency may require your attention:
* the `hound` crate (behind the `backend-file-hound` feature) uses the Apache license, see [its readme](https://github.com/ruuda/hound#license) for more details
