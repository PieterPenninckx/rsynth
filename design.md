Notes about the design
======================

The `Default` trait is not required
-----------------------------------
Implementing `Default` is sometimes not possible with `#[derive(Default)]` and it feels
awkward to implement setup (e.g. reading config files) in the `default()` method.
For `rust-vst`, an extra macro wraps the setup in a `Default` implementation, so that at least it
doesn't _feel_ awkward (but it's still a hack, of course).
Also note that `rust-vst` only requires the `Default` trait to enable a default implementation
for the `new()` function, it is not used directly by `rust-vst` itself.

Not object safe
---------------
Many of the traits are not object safe. In practice, this is not a problem for using `rust-vst`
because an extra macro wraps it.

Separate `EventHandler` trait
-----------------------------
There is a separate trait for event handling:
```rust
trait EventHandler<E> {
     fn handle_event(&mut self, event: E);
}
```
In this way, third party crates that define backends can define their own event types.
//
No associated constants for plugin meta-data
--------------------------------------------
The idea behind this was that it cannot change during the execution of the application.
We got rid of this in order to enable a more dynamic approach and in order to enable the
`Meta` trait.

Separate `AudioRenderer` and `ContextualAudioRenderer` traits
-------------------------------------------------------------
These methods were originally together with some meta-data in the `Plugin` trait,
but we have split this off so that backends can have special meta-data, without
interfering with the rendering.

Generic trait instead of generic method
---------------------------------------
The `AudioRenderer` and `ContextualAudioRenderer` traits are generic over the floating
point type, instead of having a method that is generic over _all_ float types.
In practice, backends only require renderers over f32 and/or f64, not over _all_ floating
point types. So in practice, for instance the vst backend can require
`AudioRenderer<f32>` and `AudioRenderer<f64>`. These can be implemented separately,
allowing for SIMD optimization, or together in one generic impl block.

Separate method for `set_sample_rate`
-------------------------------------
This is a separate method and not an "event type". The idea behind this is that it's guaranteed
to be called before other methods and outside of the "realtime" path (whereas
`handle_events` is called in the "realtime" path).
I don't know if this is the best solution, though. Leaving as it is until we have a more clear
understanding of it.

Decisions behind `render_buffer`
-------------------------------
`render_buffer` is at the core and some design decisions made it the way it is now.

### Push-based (instead of pull-based)
The `render_buffer` gets the buffers it needs as parameters instead of getting a queue from which
it has to "pull" the input buffers (like Jack works and if I'm not mistaken AudioUnits as well).
The upside is that it's straightforward from a developer perspective, the downside is that it's
less flexible. E.g. it's hard to implement real-time sample rate conversion in this way.
Nevertheless, I've chosen this design because it's what is convenient for most plugin developers
and developers wanting to write something like real-time sample rate conversion will probably
not use high-level abstractions like rsynth.

### Buffers as slices of slices
Somewhere an intermediate design was to have traits `InputBuffer<'a>` and `OutputBuffer<'a>`,
but this lead to a cascade of fights with the borrow checker:

* First it was problematic for the `Polyphonic` middleware (simplified pseudo-Rust of
 `Polyphonic`s `render_buffer` method):
    ```
     fn render_buffer<'a, I: InputBuffers<'a>, O: OutputBuffers<'a>>(&mut self, inputs: &I, outputs: &mut O) {
          for voice in self.voices {
              voice.render_buffer(inputs, outputs); // <-- the borrow of outputs needs to be shorter
          }
     }
     ```
     The compiler didn't allow this because the borrow of `outputs` must be shorter than the
     "external" lifetime `'a` in order to avoid overlapping borrows.
     
    * Then we implemented it as follows:
     ```rust
     fn render_buffer<I, O>(&mut self, inputs: &I, outputs: &mut O)
     where for<'a> I: InputBuffers<'a>, O: OutputBuffers<'a>
     {
         // ...
     }
     ```
     That solved one problem, but introduced `for<'a>` which is not a frequently used feature
     in Rust and which is not supported in some contexts, so I ran into some trouble with this
     (I've forgotten which).
     
For these reasons, I have abandoned this design and started using the slices instead.
This in turn gives a problem for the API-wrappers, which will want to pre-allocate the buffer
for the slices, but want to use this buffer for slices with different lifetimes.
This has been solved by the `VecStorage` struct, which has moved to its own crate.

One remaining issue is that the length of the buffer cannot be known when there are 0 inputs and
0 outputs.
I tried to solve that by having a custom data _type_ (rather than a custom _trait_): `InputChunk` and
`OutputChunk`, where `OutputChunk` is defined as follows:
```rust
struct OutputChunk<'a, 'b, S> {
     number_of_frames: usize,
     channels: &'a mut [&'b mut [S]]
}
```
Having a custom data type instead of a custom trait eliminates a number of the lifetime issues.
In order to maintain the invariant that all channels
have the same length (number_of_frames), `OutputChunk` cannot expose `channels` (because then somebody
may use the `&'a mut` reference to replace a slice with a slice of a different length.
So either this invariant needs to be given up, or `OutputChunk` needs to encapsulate everything,
but this does not give such a straightforward and easy to use API.
For this reason, I didn't keep the `OutputChunk` and continued to use the slices.

Events
------
Currently, backends that support one MIDI-port use the `Timed<RawMidiEvent>` type
and backends that support moree MIDI-ports use the `Indexed<Timed<RawMidiEvent>>` type.