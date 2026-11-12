use genco::{lang::rust::Tokens, quote};

use crate::Args;

pub fn generate_message_const_block(
    args: &Args,
    target: &str,
    protocol: u8,
    kind: u8,
    dispatch_value: u32,
) -> Tokens {
    let protocol_id = &args.protocol_id_path();
    let size = args.message_size_path();
    quote! {
        const _ASSERT_SIZE: () = {
            assert!(size_of::<$target>() == $size);
        };

        #[doc(hidden)]
        pub const DISPATCH_VALUE: u32 = $dispatch_value;
        pub const PREPARED: Self = {
            let mut bytes = [0u8; core::mem::size_of::<Self>()];
            bytes[0] = 1;
            bytes[1] = 1;
            bytes[2] = 1;
            bytes[3] = 1;
            let mut value = unsafe { core::mem::transmute::<[u8; core::mem::size_of::<Self>()], Self>(bytes) };
            value.protocol = $protocol_id::new_u8($protocol).unwrap();
            value.kind = $kind;
            value
        };
    }
}
