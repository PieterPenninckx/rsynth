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
//! works because Rust does not yet support specialization (at the time of writing):
//! the following will not compile.
//! ```compile_fail
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
//! you want to handle in a special way. These techniques are supported by the [`syllogism`] crate.
//! Additionally, we advise to provide also support using specialization:
//!
//! In the `Cargo.toml` file:
//! ```toml
//! [features]
//! default=["stable"]
//! stable=["syllogism", "syllogism-macro"]
//!
//! [dependencies]
//! syllogism = {version = "0.1", optional = true}
//! syllogism-macro = {version = "0.1", optional = true}
//! ```
//!
//! In your source code:
//! ```
//! #[cfg(not(feature = "stable"))]
//! impl<E, P> EventHandler<E> for MyMiddleware<P>
//! where // ...
//! # P: EventHandler<E> , E: IsNot<SpecialEventType>
//! {
//!     // An implementation that uses the syllogism crate.
//! #    fn handle_event(&mut self, event: E) {
//! #        self.child.handle_event(event);
//! #    }
//! }
//!
//! #[cfg(not(feature = "stable"))]
//! impl<E, P> EventHandler<E> for MyMiddleware<P>
//! where // ...
//! # P: EventHandler<E> , E: IsNot<SpecialEventType> {
//!     // An implementation that uses specialization.
//! #    fn handle_event(&mut self, event: E) {
//! #        self.child.handle_event(event);
//! #    }
//! }
//! ```
//!
//! ### Specializing for events with a concrete type
//!
//! If the event type for which you want to specialize is a concrete type,
//! you can use the [`IsNot`] trait from the [`syllogism`] crate to distinguish the generic
//! types from the special type.
//! Because no event type should implement `IsNot<Self>`, the compiler
//! knows there is no overlap. All event types should implement `IsNot<T>` for all
//! other types `T`. How this is achieved, is explained below.
//!
//! ```
//! use rsynth::event::EventHandler;
//! use syllogism::IsNot;
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
//! no type (even not in a dependent crate) will implement `IsNot<Self>`.
//! Not implementing `IsNot<Self>` is just a convention,
//! it is not compiler-enforced and the compiler cannot see
//! this. To work around this, you can use the [`Specialize`] trait from the
//! [`syllogism`] crate:
//!
//! ```
//! use rsynth::event::EventHandler;
//! use syllogism::{Specialize, Distinction};
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
//! so that we can link to it in the documentation of `rsynth`.
//!
//! Writing events
//! ==============
//! 
//! Copy
//! ----
//!
//! If possible, implement the `Copy` trait for the event,
//! so that the `Polyphonic` middleware can dispatch this event to all the voices.
//!
//! Compatibility
//! -------------
//!
//! In order to allow specialization for the middleware (see above),
//! any event type `T1` needs to implement
//!
//! * `IsNot<T2>` for every event type `T2` that differs from `T1`
//! * `Specialize<T>` for every event type `T`, including `T1` itself.
//!
//! Here "every event type" also needs to include event types defined in other crates that you may
//! or may not know about. In order to make this possible, open an issue for `rsynth` announcing
//! that you want to develop a crate and we can provide you
//!
//! * a trait that is implemented for all event types defined outside your crate
//! * a macro that you should use for all event types defined in your crate.
//!
//! In the meanwhile, you can use the trait `NotInUnpublishedCrate` and the macro
//! `macro_for_unpublished_crate`.
//!
//! Please note that there are some subtleties around defining events with conditional compilation.
//! If you are planning to define some events only when some compile-time conditions are met,
//! please state so in the issue and we can discuss how we will handle this.
//!
//! ### Compatibility in the manual way
//!
//! You can manually implement `IsNot` and `Specialize` as follows.
//! We include this documentation for clarity; in practice, you may want
//! to use the macro's as explained below in order to lower the risk that
//! you forget something.
//!
//! #### Implementing `IsNot` by hand
//!
//! If you declare more than one event type, you need to ensure that each
//! event type implements `IsNot` of each other. Suppose that you can
//! use the trait `NotInMyCrate` and the macro `macro_for_my_crate`,
//! then you can ensure by hand that `IsNot` is implemented:
//! ```
//! use syllogism::IsNot;
//! # trait NotInMyCrate {}
//! # macro_rules! macro_for_my_crate {
//! # ($($t:tt)*) => {}
//! # }
//! struct EventType1 {}
//! struct EventType2 {}
//! struct EventType3 {}
//!
//! impl IsNot<EventType2> for EventType1 {}
//! impl IsNot<EventType3> for EventType1 {}
//! impl<T> IsNot<T> for EventType1 where T: NotInMyCrate {}
//! macro_for_my_crate!(impl trait for EventType1);
//!
//! impl IsNot<EventType1> for EventType2 {}
//! impl IsNot<EventType3> for EventType2 {}
//! impl<T> IsNot<T> for EventType2 where T: NotInMyCrate {}
//! macro_for_my_crate!(impl trait for EventType2);
//!
//! impl IsNot<EventType1> for EventType3 {}
//! impl IsNot<EventType2> for EventType3 {}
//! impl<T> IsNot<T> for EventType3 where T: NotInMyCrate {}
//! macro_for_my_crate!(impl trait for EventType3);
//! ```
//!
//! #### Implementing `Specialize` by hand
//!
//! Each event type needs to implement `Specialize<Self>`.
//! Additionally, if you declare more than one event type, you need to ensure that each
//! event type implements `Specialize` of each other.
//! Suppose that you can
//! use the trait `NotInMyCrate` and the macro `macro_for_my_crate`, then you can ensure by hand
//! that `Specialize` is implemented:
//! ```
//! use syllogism::{Specialize, Distinction};
//! # trait NotInMyCrate {}
//! # macro_rules! macro_for_my_crate {
//! # ($($t:tt)*) => {}
//! # }
//! struct EventType1 {}
//! struct EventType2 {}
//! struct EventType3 {}
//!
//! impl Specialize<EventType1> for EventType1 {
//!     fn specialize(self) -> Distinction<EventType1, Self> {
//!         Distinction::Special(self)
//!     }
//! }
//! impl Specialize<EventType2> for EventType1 {
//!     fn specialize(self) -> Distinction<EventType2, Self> {
//!         Distinction::Generic(self)
//!     }
//! }
//! impl Specialize<EventType3> for EventType1 {
//!     fn specialize(self) -> Distinction<EventType3, Self> {
//!         Distinction::Generic(self)
//!     }
//! }
//! impl<T> Specialize<T> for EventType1 where T: NotInMyCrate {
//!     fn specialize(self) -> Distinction<T, Self> {
//!         Distinction::Generic(self)
//!     }
//! }
//! macro_for_my_crate!(impl trait for EventType1);
//!
//! // And similar for `EventType2` and `EventType3`, I'm omitting this for brevity.
//! ```
//!
//! ### Compatibility by using the macros from `syllogism-macro`
//!
//! Most of the implementations of `IsNot` and `Specialize` can be done by using
//! the macro [`impl_specialization`] from the `syllogism-macro` crate:
//! under the same assumptions as above, this can be simplified
//! to the following (but see the note below for types with type parameters):
//! ```
//! use syllogism_macro::impl_specialization;
//! # trait NotInMyCrate {}
//! # macro_rules! macro_for_my_crate {
//! # ($($t:tt)*) => {}
//! # }
//! struct EventType1 {}
//! struct EventType2 {}
//! struct EventType3 {}
//!
//! impl_specialization!(
//!     trait NotInMyCrate;
//!     macro macro_for_my_crate;
//!
//!     type EventType1;
//!     type EventType2;
//!     type EventType3;
//! );
//! # fn main () {}
//! ```
//! The caveats are that for types with type parameters, you still need to implement `Specialize`
//! by hand and if you have more types with type parameters, you must use different names for
//! the type parameters in the call to the [`impl_specialization`] macro.
//! For more information, see the documentation of [`impl_specialization`].
//!
//! [`VecStorage` and `VecStorageMut`]: ./vecstorage/index.html
//! [`Transparent`]: ./transparent/trait.Transparent.html
//! [`EventHandler`]: ../event/trait.EventHandler.html
//! ["Writing events" below]: ./index.html#writing-events
//! [`syllogism`]: https://docs.rs/syllogism/0.1.0/syllogism/
//! [`IsNot`]: https://docs.rs/syllogism/0.1.0/syllogism/trait.IsNot.html
//! [`Specialize`]: https://docs.rs/syllogism/0.1.0/syllogism/trait.Specialize.html
//! [`impl_specialization`]: https://docs.rs/syllogism-macro/0.1.0/syllogism_macro/macro.impl_specialization.html
pub mod vecstorage;
pub mod transparent;

#[cfg(feature = "stable")]
#[macro_use]
pub mod compatibility;