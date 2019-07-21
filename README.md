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

The documentation can be found
* [on docs.rs](https://docs.rs/rsynth/) for the version that is distributed via crates.io.
* [on GitHub pages](https://pieterpenninckx.github.io/rsynth) for an irregularly updated documentation of the master branch
* on your local machine after running `cargo rustdoc --features backend-jack,backend-vst` for the most up-to-date documentation 

# Examples
There are full examples in 
[the examples folder in the source code](https://github.com/PieterPenninckx/rsynth/tree/master/examples).


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

Features that are likely going to be postponed for a long time, depending on the capacity of the
team and other issues (unless somebody joins to help with these)

- Support for LV2

In the long term, rsynth can be split into multiple crates for maximum reusability
and for license clarity (e.g. when one back-end requires a different license).
We're currently keeping everything together because it's easier to coordinate breaking changes
over the various components in this way.

# Contributing

Contributions and suggestions are welcome.

In order to avoid pull requests from being broken by other changes, please open an issue or
have an issue assigned to you before you start working on something. 
In this way, we can coordinate development.
Issues labeled with "good first issue" should not conflict too much with other changes
that are in flight, but better check before you start working on one.

Pull requests should be opened against the `master` branch.

## Testing

In order to run all tests, run the following:
```bash
cargo test --features backend-jack,backend-vst
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
