Version 0.1.1
=============
* Clarify copyright (MIT/3 clause BSD) (synchronise Cargo.toml with README.md).
* Deprecate envelopes.
* Polyphony has been deprecated in favour of the `polyphony` crate.
* Move from the `sample` crate to `dasp_sample` to reduce compile times a little.
* Re-export jack-specific data types to make it easier to upgrade `jack` without breaking things.
* Give access to jack backend in callbacks.
* Implement `Display` and `Error` for `HoundAudioError` and for `CombinedError`.

Version 0.1.0
=============
* Add support for jack as a backend
* Add an abstraction for audio buffers
* Add support for offline rendering
* Add a backend independent mechanism for specifying meta-data. It also has a “user friendly” way for specifying the meta-data.
* Lots of edits to the documentation
* Lots and lots of refactoring big and small

Version 0.0.1
=============
Initial release.
