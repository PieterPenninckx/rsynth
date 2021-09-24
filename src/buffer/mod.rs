//! Audio buffers.

pub trait DelegateHandling<P, D> {
    type Output;
    fn delegate_handling(&mut self, p: &mut P, d: D) -> Self::Output;
}

/// Call all the backend-specific macro's for a given struct.
///
/// # Example
/// ```
/// use rsynth::derive_ports;
/// use rsynth::event::{Timed, RawMidiEvent, CoIterator};
/// #[cfg(feature = "backend-jack")]
/// use rsynth::derive_jack_port_builder;
///
///
/// derive_ports! {
///     struct MyPorts<'a> {
///         audio_in: &'a [f32],
///         audio_out: &'a mut [f32],
///         midi_in: &'a mut dyn Iterator<Item = Timed<RawMidiEvent>>,
///         midi_out: &'a mut dyn CoIterator<Item = Timed<RawMidiEvent>>,
///     }
///
///     #[cfg(feature = "backend-jack")]
///     derive_jack_port_builder! {
///         struct MyPortsBuilder {
///         }
///     }
/// }
/// ```
///
/// # The struct
/// The `derive_ports!` macro expects exactly one struct (`MyPorts` in the example above).
///
/// Note that the struct cannot currently be generic, except for one lifetime parameter.
/// The struct can have an arbitrary number of meta-attributes defined (e.g. `#[derive(Clone)]`).
///
/// The following table describes what field types are currently supported by which backend.
/// | Field type        |  Meaning  | Jack via [`jack`] |
///  |-------------------|-----------|:-----------------:|
///  | `&'a [f32]`         | Audio in  |        ✓          |
///  | `&'a [f32]`         | CV in     |        ✘          |
///  | `&'a mut [f32]`     | Audio out |        ✓          |
///  | `&'a mut [f32]`     | CV out    |        ✘          |
///  | `&'a mut dyn Iterator<Item = Timed<RawMidiEvent>`  | Midi in | ✓ |
///  | `&'a mut dyn CoIterator<Item = Timed<RawMidiEvent>` | Midi out | ✓ |
///
/// # The backend-specific macros
/// The `derive_ports!` macro expects an arbitrary number of backend-specific macro's.
/// Before each backend-specific macro, you can specify an arbitrary number of attributes.
///
/// See the documentation of the backend-specific macro's for more information:
/// * [`derive_jack_port_builder`]
#[macro_export]
macro_rules! derive_ports {
    // This rule matches the original input.
    (
        $(#[$global_meta:meta])*
        struct $buffer_name:ident$(<$lt:lifetime>)?
        {
            $($global:tt)*
        }
        $(
            $(#[$local_meta:meta])*
            $local_macro:ident!{
                $($local_token:tt)*
            }
        )*
    ) => {
        $(#[$global_meta])*
        pub struct $buffer_name$(<$lt>)?
        {
            $($global)*
        }
        derive_ports!{
            @inner
            $buffer_name
            @($($global)*)
            @(
                $(
                    $(#[$local_meta])*
                    $local_macro!{
                        @()
                        @($($local_token)*)
                    }
                )*
            )
        }
    };
    // No tokens need to be processed anymore, this is the end of this expansion.
    (
        @inner
        $buffer_name:ident
        @()
        @(
            $(
                $(#[$local_meta:meta])*
                $local_macro:ident!{
                    @($($global_processed_static:tt)*)
                    @($($local_token:tt)*)
                }
            )*
        )
    ) => {
        $(
            $(#[$local_meta])*
            $local_macro!{
                @($($local_token)*)
                @($($global_processed_static)*)
                $buffer_name
            }
        )*
    };
    // Replace a lifetime by a static lifetime and continue processing the remaining tokens.
    (
        @inner
        $buffer_name:ident
        @($global_head:lifetime $($global_tail:tt)*)
        @(
            $(
                $(#[$local_meta:meta])*
                $local_macro:ident!{
                    @($($global_processed_static:tt)*)
                    @($($local_token:tt)*)
                }
            )*
        )
    ) => {
        derive_ports!{
            @inner
            $buffer_name
            @($($global_tail)*)
            @(
                $(
                    $(#[$local_meta])*
                    $local_macro!{
                        @($($global_processed_static)* 'static)
                        @($($local_token)*)
                    }
                )*
            )
        }
    };
    // Since the previous rule didn't match, the token is not a lifetime.
    // Just pass it "as is" and continue processing the remaining tokens.
    (
        @inner
        $buffer_name:ident
        @($global_head:tt $($global_tail:tt)*)
        @(
            $(
                $(#[$local_meta:meta])*
                $local_macro:ident!{
                    @($($global_processed_static:tt)*)
                    @($($local_token:tt)*)
                }
            )*
        )
    ) => {
        derive_ports!{
            @inner
            $buffer_name
            @($($global_tail)*)
            @(
                $(
                    $(#[$local_meta])*
                    $local_macro!{
                        @($($global_processed_static)* $global_head)
                        @($($local_token)*)
                    }
                )*
            )
        }
    };
}
