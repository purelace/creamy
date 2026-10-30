use genco::{lang::rust::Tokens, quote};

pub fn generate_message_const_block(
    size: &str,
    target: &str,
    group: u8,
    kind: u8,
    dispatch_value: u32,
) -> Tokens {
    quote! {
        const _ASSERT_SIZE: () = {
            assert!(size_of::<$target>() == $size);
        };

        #[doc(hidden)]
        pub const DISPATCH_VALUE: u32 = $dispatch_value;
        pub const PREPARED: Self = {
            let mut value = unsafe { core::mem::zeroed::<Self>() };
            value.group = $group;
            value.kind = $kind;
            value
        };
    }
}
