//! Utilities to be used when developing backends and middleware.
//! # `VecStorage` and `VecStorageMut`
//! Struct to be able to re-use the storage of a vector
//! for borrowing values with different lifetimes.
//!
//! ## Examples
//! The following code does not compile:
//! ```ignore
//! let mut v = Vec::with_capacity(2);
//! {
//!     let x = 1; let y = 2;
//!     v.push(&x);
//!     v.push(&y);
//!     v.clear(); // We stop borrowing here, but the compiler doesn't know that.
//! }
//! {
//!     let a = 1; let b = 2;
//!     v.push(&a);
//!     v.push(&b);
//!     v.clear(); // We stop borrowing here, but the compiler doesn't know that.
//! }
//! ```
//!
//! You can use `VecStorage` to solve this problem:
//! ```
//! use rsynth::backend::utilities::VecStorage;
//! let mut v = VecStorage::with_capacity(2);
//! {
//!     let x = 1; let y = 2;
//!     let mut guard = v.vec_guard();
//!     // Now guard behaves like a vector.
//!     guard.push(&x); // No memory allocation here, we use the memory allocated in `v`.
//!     guard.push(&y);
//!     // If we were going to push more items on the guard, we would allocate memory.
//!     // When guard goes out of scope, it is cleared.
//! }
//! {
//!     let a = 1; let b = 2;
//!     let mut guard = v.vec_guard();
//!     // Now guard behaves like a vector.
//!     guard.push(&a);
//!     guard.push(&b);
//!     // When guard goes out of scope, it is cleared.
//! }
//! ```
//!
//! `VecStorage<T>` allocates memory just like `Vec<&T>`,
//! but it does not borrow anything.
//! You can create a `VecGuard` with the `vec_guard` method.
//! The `VecGuard` uses the memory from the `VecStorage` and can temporarily
//! be used just like a `Vec<&T>`
//! (i.e.: it implements `Deref<Target=Vec<&T>>` and `DerefMut<Target=Vec<&T>>`)
//! When the `VecGuard` is dropped, the memory "goes back to the `VecStorage`" and
//! can be re-used later on to store references with a different lifetime.
//!
//! `VecStorageMut<T>` is similar: it allows you to create a `VecGuardMut`, which
//! can be used just like a `Vec<&mut T>`.
pub mod vecstorage;
pub mod is_not;
pub mod specialize;