//! Utilities to be used when developing backends and middleware.
//!
//! Writing a backend
//! =================
//!
//! Custom backends do not need to be in the `rsynth` crate, you can write
//! a backend in a separate crate. If you encounter problems that prevent you
//! from writing your backend in a separate crate (e.g., we have forgotten to
//! mark something as `pub`), let us know by opening an issue.
//!
//! Creating an input buffer and an output buffer
//! ---------------------------------------------
//!
//! When you pass `&[&[f32]]` for the input buffer and `&mut[&mut[f32]]`
//! for the output buffer, you may face the challenge that you can have
//! the buffers for each channel and you can `collect()` them into a `Vec`,
//! but you don't want to allocate that `Vec` in the real-time thread.
//! In order you to help overcome this problem, we provide
//! [`VecStorage` and `VecStorageMut`], which you can pre-allocate and re-use for every
//! call to `render_buffer` with different lifetimes of the slices.
//!
//! Writing a custom trait for a backend
//! ------------------------------------
//!
//! When the backend needs a special trait to be implemented by the plugin,
//! ideally all middleware should "pass trough" this trait. The middleware
//! does this by implementing the [`Transparent`] trait. The backend needs
//! to be able to "look trough" the middleware. This can be achieved by using
//! a blanket impl as follows:
//! ```
//! use rsynth::dev_utilities::transparent::Transparent;
//! trait MyCustomTrait {
//!     // ...
//! }
//!
//! impl<T> MyCustomTrait for T
//! where
//!    T: Transparent,
//!    <T as Transparent>::Inner: MyCustomTrait,
//! {
//!     // ...
//! }
//! ```
//!
//! Writing custom events
//! ---------------------
//!
//! See ["Writing events" below].
//!
//! Publishing a backend crate
//! --------------------------
//!
//! When you publish a backend crate, let us know by opening an issue or pull request
//! so that we can link to it in the documentation of rsynth.
//!
//!
//! Writing events
//! ==============
//!
//! Implement `Copy` if possible
//! ----------------------------
//!
//! If possible, implement the `Copy` trait for the event,
//! so that the `Polyphonic` middleware can dispatch this event to all the voices.
//!
//!
//! [`VecStorage` and `VecStorageMut`]: ./vecstorage/index.html
//! ["Writing events" below]: ./index.html#writing-events
pub mod vecstorage;
