//! Audio buffers.

pub trait DelegateHandling<P, D> {
    type Output;
    fn delegate_handling(&mut self, p: &mut P, d: D) -> Self::Output;
}

#[macro_export]
macro_rules! derive_ports {
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
                @($(#[$local_meta])*)
                @($($global_processed_static)*)
                $buffer_name
            }
        )*
    };
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
