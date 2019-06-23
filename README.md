# rsynth

A crate for developing audio plugins and applications in Rust, with a focus on software synthesis.
Rsynth is well suited as a bootstrap for common audio plugin generators. 
It handles voices, voice-stealing, polyphony, etc. so the programmer's main focus can be DSP. 

Rsynth has the following components:

* An API abstraction layer
* Glue code for different API's (called back-ends). Currently supported are
  * [rust-vst](https://github.com/RustAudio/vst-rs)
  * Jack
* Middleware components that you can put between your code and the abstraction layer to provide 
  various functionalities:
  * polyphony
  * ...

# Documentation
[Documentation](https://resamplr.github.io/rsynth)

# Examples
There are full examples in 
[the examples folder in the source code](https://github.com/resamplr/rsynth/tree/master/examples).

Below we give a simplified example that illustrates the main features.
```rust
struct MyPlugin {
    // ...
}

impl<C, H> Plugin<C> for MyPlugin
where
    C: WithHost<H>,
    H: HostInterface
{
    // For brevity, we omit some methods that describe the plugin (plugin name etc.)

    fn set_sample_rate(&mut self, sample_rate: f64) {
        // ...
    }

    fn render_buffer<F>(&mut self, inputs: &[&[F]], outputs: &mut [&mut [F]], context: &mut C)
    where
        F: Float + AsPrim,
    {
        // This is the core of the plugin. Here you do the actual sound rendering.
        // `inputs` is a slice of input buffers. 
        // `outputs` is a slice of output buffers.
        // An input buffer is just a slice of floats (f32 or f64),
        // an output buffer is just a mutable slice of floats.
        // `context` contains some data that is computed outside of the `MyPlugin` struct.
        // In this case, because it implements `WithHost`, we can access the host as follows:
        let host = context.host();
    }
}

impl<C> EventHandler<Timed<RawMidiEvent>, C> for MyPlugin
{
    fn handle_event(&mut self, timed: Timed<RawMidiEvent>, context: &mut C) {
        // Here we can handle the event.
    }
}
```

This plugin can then be used in the main function as follows:
```rust
fn main() {
    let my_plugin = MyPlugin{ /* ... */ };
    // Use the `ZeroInit` middleware:
    let zero_initialized = ZeroInit::new(my_plugin);
    // this may be wrapped further in other middleware.
    
    jack_backend::run(zero_initialized);
}
```

# Current State

rsynth is in its early stage of development and many changes are breaking changes.
There is currently no support for GUI's.
The team behind it is very small, so progress is slow.

# Roadmap

We try to focus on features that we are actually using ourselves.
This helps to ensure that the features that we provide, can actually be used in practice.
So if you want to use a particular feature that isn't there yet, feel free to open an issue (if
needed) and you can volunteer to test the feature before it is merged. 

Features that are likely to be realized:

- Add a back-end for testing
- Add middleware to split the audio-buffer so that timed events are at sample `0`
- Add support for envelopes

Features that are likely going to be postponed for a long time, depending on the capacity of the
team and other issues (unless somebody joins to help with these)

- Support for LV2

In the long term, rsynth can be split into multiple crates for maximum reusability
and for license clarity (e.g. when one back-end requires a different license).
We're currently keeping everything together because it's easier to coordinate breaking changes
over the various components in this way.

# Testing

We're currently using the [syllogism](https://crates.io/crates/syllogism) crate as a workaround
for the lack of specialization in Rust, but you can also test whether it still works when using the
`specialization` feature using a feature flag with nightly rust.

For this reason, it's advised to run the tests twice: one with stable Rust, using the 
`syllogism` crate:

```bash
cargo test --features jack-backend,vst-backend
```

and once using nightly rust, without the `syllogism` crate and with the `specialization` feature:

```bash
cargo +nightly test --no-default-features --features jack-backend,vst-backend
```

# Contributing

Contributions and suggestions are welcome.

In order to avoid pull requests from being broken by other changes, please open an issue or
have an issue assigned to you before you start working on something. 
In this way, we can coordinate development.
Issues labeled with "good first issue" should not conflict too much with other changes
that are in flight, but better check before you start working on one.

## Code formatting
Please use `cargo fmt` to format your code before opening a pull request.

_Tip_: you can auto-format your code on save in your IDE:
* IntelliJ: `File > Settings > Languages & Frameworks > Rust > Rustfmt > Run rustfmt on save`
* [Visual Studio Code with `rls-vscode`](https://github.com/rust-lang/rls-vscode#format-on-save)

# Sponsorship

rsynth is helped by [Resamplr.com](https://resamplr.com/), a virtual instrument website.

# License 

MIT/BSD-3 License
