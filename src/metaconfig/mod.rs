macro_rules! match_number {
    (@with_number ($number:expr) @ ) => {
        _ => {unreachable!();}
    },
    (@with_number ($number:expr) @ $in_name:expr,) => {
        $number => $in_name,
    },
    (@with_number ($number:expr) @ $in_name_head:expr, $($in_name_tail:expr,)) => {
        $number => $in_name,
        audio_in_name!(($number+1) @ $($in_name_tail,)*);
    },
    ($($)*)
}
macro_rules! define {
    (impl $plugin:ty; $name_head:ident : $definition_head:tt $(, $name_tail:ident : $definition_tail:tt )*) => {
        define!(impl $plugin; , $name_head:$definition_head $(,$name_tail:$definition_tail)*);
    },
    (impl $plugin:ty; , name : $definition_head:expr $(, $name_tail:ident : $definition_tail:tt )*) => {
        impl $crate::CommonPluginMeta for $plugin {
            fn name(&self) -> &str { $definition_head }
        }
        
        define!(impl $plugin:ty; $(,$name_tail: $definition_tail));
    },
    (impl $plugin:ty; , audio : { in: {$($in_name:expr,)*}, out: {$($out_name:expr,)*} } $(, $name_tail:ident : $definition_tail:tt )*) => {
        impl $crate::CommonAudioPortMeta for $plugin {
            fn audio_input_name(&self, index: usize) -> &str {
                match index {
                    $()*
                }
            }
        }
        
        define!(impl $plugin:ty; $(,$name_tail: $definition_tail));
    },
}
