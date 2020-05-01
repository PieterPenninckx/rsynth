# rsynth

A crate for developing audio plugins and applications in Rust, with a focus on software synthesis.
Rsynth is well suited as a bootstrap for common audio plugin generators. 
It handles voices, voice-stealing, polyphony, etc. so the programmer's main focus can be DSP.
It is currently most suitable for real-time or "streaming" audio processing.
E.g. you cannot use it to reverse audio in time.

Rsynth has the following components:

* An API abstraction layer
* Glue code for different API's (called back-ends). Currently supported are
  * [rust-vst](https://github.com/RustAudio/vst-rs)
  * [Jack](https://crates.io/crates/jack)
  * offline audio rendering (from/to `.wav` and `.mid` files)
* Structs that provide various functionalities:
  * polyphony
  * ...

It has the following features
* Duplex audio input and output
* Midi input and output
* Basic meta-data

# Documentation

The documentation can be found
* [on docs.rs](https://docs.rs/rsynth/) for the version that is distributed via crates.io.
* [on GitHub pages](https://pieterpenninckx.github.io/rsynth/rsynth) for an irregularly updated documentation of the master branch
* on your local machine after running `cargo rustdoc --features all` for the most up-to-date documentation 

# Examples
There are full examples in 
[the examples folder in the source code](https://github.com/PieterPenninckx/rsynth/tree/master/examples).


# Current State

`rsynth` is in its early stage of development and many changes are breaking changes.
There is currently no support for GUI's.
The team behind it is very small, so progress is slow.

# Roadmap

We try to focus on features that we are actually using ourselves.
This helps to ensure that the features that we provide, can actually be used in practice.
So if you want to use a particular feature that isn't there yet, feel free to open an issue (if
needed) and you can volunteer to test the feature before it is merged. 

In the long term, `rsynth` can be split into multiple crates for maximum re-usability
and for license clarity (e.g. when one back-end requires a different license).
We're currently keeping everything together because it's easier to coordinate breaking changes
over the various components in this way.

# Contributing

Contributions and suggestions are welcome!

## Opening and voting for issues

If there is a feature you would like to see, feel free to open an issue or "vote" for an issue by
adding the "thumbs up" emoji.

## Reviewing pull requests

Two pair of eyes see more than just one. Have a look at 
[this issue](https://github.com/PieterPenninckx/rsynth/issues/74) if you want to help by reviewing
code.

## Writing blog posts

If you write a plugin that uses `rsynth`, why not share your experience by writing a blog post?
(Make sure you clearly indicate the date and the particular version you are trying.)

## Updating documentation

Everybody loves good documentation, but it's a lot of work to write and maintain.
Contributing to the doc comments is a way to contribute that does not require that many
skills, but which has a big impact.
For practical aspects, see "Contributing code" below.

## Contributing code

Code contributions are certainly welcome as well!

In order to avoid pull requests from being broken by other changes, please open an issue or
have an issue assigned to you before you start working on something. 
In this way, we can coordinate development.
Issues labeled with "good first issue" should not conflict too much with other changes
that are in flight, but better check before you start working on one.

Don't worry if you only have a partial solution. You can still open a pull request for that. 
You can definitely split the solution for an issue into different pull requests. 

I tend to squash all commits, which means that all your intermediate commits are combined into
one commit. This has the advantage that you don't need to worry about what is in these intermediate
commits. On the other hand, some people want to have more activity on their GitHub timeline. If
you don't want me to squash the commits, let me know when you open the pull request.

Pull requests should be opened against the `master` branch. 

## Testing

In order to run all tests, run the following:
```bash
cargo test --features all
```

If you have trouble running this locally because you do not have jack-related libraries installed,
no worries: you can still open a pull request; this will automatically trigger a CI build that runs
all tests for you.

## Code formatting
Please use `cargo fmt` to format your code before opening a pull request.

_Tip_: you can auto-format your code on save in your IDE:
* IntelliJ: `File > Settings > Languages & Frameworks > Rust > Rustfmt > Run rustfmt on save`
* [Visual Studio Code with `rls-vscode`](https://github.com/rust-lang/rls-vscode#format-on-save)

# Sponsorship

Alexander Lozada's contributions to rsynth are helped by [Resamplr.com](https://resamplr.com/), a virtual instrument website.

# License 

The source code of `rsynth` is licensed under the MIT/BSD-3 License.

Note that in order to use `rsynth` in combination with other crates (libraries), the combined work needs
to comply with the license of that crate as well. In particular, the following optional dependencies may require your attention:
* the `hound` crate (behind the `backend-file-hound` feature) uses the Apache license, see [its readme](https://github.com/ruuda/hound#license) for more details
