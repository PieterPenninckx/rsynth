//! Utilities to be used when developing backends and middleware.
//!
//! Writing a backend
//! =================
//!
//! Custom backends do not need to be in the `rsynth` crate, you can write
//! a backend in a separate crate. If you encounter problems that prevent you
//! from writing your backend in a separate crate (e.g., we have forgotter to
//! mark something as `pub`), let us know by opening an issue.
//!
//! Creating an input buffer and an output buffer
//! ---------------------------------------------
//!
//! When you pass `*[&[f32]]` for the input buffer and `&mut[&mut[f32]]`
//! for the output buffer, you may face the challenge that you can get
//! the buffers for each channel and you can `collect()` them into a `Vec`,
//! but you don't want to allocate that `Vec` in the realtime thread.
//! In order you to help overcome this problem, we provide the `VecStorage`
//! and `VecStorageMut`, which you can pre-allocate and re-use for every
//! call to `render_buffer` with different lifetimes of the slices.
//!
//! Writing a custom trait
//! ----------------------
//!
//! When the backend needs a special trait to be implemented by the plugin,
//! ideally all middleware should "pass trough" this trait. The middleware
//! does this by implementing the `Transparent` trait. The backend needs
//! to be able to "look trough" the middleware. This can be achieved by using
//! a blanket impl as follows:
//! ```
//! impl<T> MyCustomTrait for T
//! where
//!    T: Transparent,
//!    <T as Transparent>::Inner: VstPlugin,
//! {
//!     // ...
//! }
//! ```
//!
//! Writing custom events
//! ---------------------
//!
//! See below.
//!
//! Publishing a backend crate
//! --------------------------
//!
//! When you publish a backend crate, let us know, so that we can link to it in
//! our documentation.
//!
//! Writing middleware
//! ==================
//!
pub mod vecstorage;
#[macro_use]
pub mod specialize;
#[macro_use]
pub mod compatibility;

