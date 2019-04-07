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
//! Writing middleware
//! ==================
//!
//! Implementing backend-specific traits
//! ------------------------------------
//!
//! Some backends might require plugins to implement a trait specific for that
//! backend. In order to implement this trait for the middleware as well,
//! you can simply implement the [`Transparent`] trait. A blanket impl defined
//! by the backend will then ensure that the middleware also implements the
//! backend specific trait.
//!
//! Handling events
//! ---------------
//!
//! Middleware needs to implement [`EventHandler`] for "all" events.
//! If the middleware does not do anything with events at all, it is
//! easy to simply pass it to the child:
//! ```
//! use rsynth::event::EventHandler;
//! struct MyMiddleware<P> {
//!     child: P
//! }
//! impl<E, P> EventHandler<E> for MyMiddleware<P>
//! where P: EventHandler<E> {
//!     fn handle_event(&mut self, event: E) {
//!         self.child.handle_event(event);
//!     }
//! }
//! ```
//!
//! When trying to handle one event type in a special way, this no longer
//! works because Rust does not support specialization (at the time of writing):
//! the following will not compile.
//! ```ignore
//! use rsynth::event::EventHandler;
//! struct MyMiddleware<P> {
//!     child: P
//! }
//! impl<E, P> EventHandler<E> for MyMiddleware<P>
//! where P: EventHandler<E> {
//!     fn handle_event(&mut self, event: E) {
//!         self.child.handle_event(event);
//!     }
//! }
//! # struct SpecialEventType {}
//! impl<P> EventHandler<SpecialEventType> for MyMiddleware<P>
//! {
//!     fn handle_event(&mut self, event: SpecialEventType) {
//!         // Do something specific with the middleware.
//!     }
//! }
//! ```
//!
//! You can solve this problem in two ways, depending on the type that
//! you want to handle in a special way.
//!
//! ### Specializing for events with a concrete type
//!
//! If the event type for which you want to specialize is a concrete type,
//! you can use the [`IsNot`] trait to distinguish the generic types from the special
//! type. Because no event type should implement `IsNot<Self>`, the compiler
//! knows there is no overlap. All event types should implement `IsNot<T>` for all
//! other types `T`. How this is achieved, is explained below.
//!
//! ```
//! use rsynth::event::EventHandler;
//! use rsynth::dev_utilities::specialize::IsNot;
//! struct MyMiddleware<P> {
//!     child: P
//! }
//! # struct SpecialEventType {}
//!
//! // The generic event types
//! impl<E, P> EventHandler<E> for MyMiddleware<P>
//! where P: EventHandler<E> , E: IsNot<SpecialEventType> {
//!     fn handle_event(&mut self, event: E) {
//!         self.child.handle_event(event);
//!     }
//! }
//!
//! // The special event type
//! impl<P> EventHandler<SpecialEventType> for MyMiddleware<P>
//! {
//!     fn handle_event(&mut self, event: SpecialEventType) {
//!         // Do something specific with the middleware.
//!     }
//! }
//! ```
//!
//!
//! ### Specializing for events of a type parameter
//!
//! If the event type for which you want to specialize is a type parameter,
//! you cannot use the `IsNot` trait because the compiler cannot know that
//! no type (even not in a dependent crate) will event `IsNot<Self>`. This is
//! just a convention, it is not compiler-enforced and the compiler cannot see
//! this. To work around this, you can use the [`Specialize`] trait:
//!
//! ```
//! use rsynth::event::EventHandler;
//! use rsynth::dev_utilities::specialize::{Specialize, Distinction};
//! struct MyMiddleware<P> {
//!     child: P
//! }
//! # struct SpecialEventType {}
//! # impl Specialize<SpecialEventType> for SpecialEventType {
//! #   fn specialize(self) -> Distinction<SpecialEventType, Self> { Distinction::Special(self) }
//! # }
//!
//! impl<E, P> EventHandler<E> for MyMiddleware<P>
//! where P: EventHandler<E> , E: Specialize<SpecialEventType> {
//!     fn handle_event(&mut self, event: E) {
//!         match event.specialize() {
//!             Distinction::Special(special) => {
//!                 // Do something special
//!             },
//!             Distinction::Generic(generic) => {
//!                 // self.child.handle_event(generic);
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! Writing special events for the middleware
//! -----------------------------------------
//!
//! See ["Writing events" below].
//!
//! Publishing a middleware crate
//! -----------------------------
//!
//! When you publish a middleware crate, let us know by opening an issue or pull request
//! so that we can link to it in the documentation of rsynth.
//!
//! Writing events
//! ==============
//!
//! [`VecStorage` and `VecStorageMut`]: ./vecstorage/index.html
//! [`Transparent`]: ./transparent/trait.Transparent.html
//! [`EventHandler`]: ../event/trait.EventHandler.html
//! [`IsNot`]: ./specialize/trait.IsNot.html
//! [`Specialize`]: ./specialize/trait.Specialize.html
//! ["Writing events" below]: ./index.html#writing-events
pub mod vecstorage;
pub mod transparent;
#[macro_use]
pub mod specialize;
#[macro_use]
pub mod compatibility;
