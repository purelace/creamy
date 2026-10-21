use std::borrow::Cow;

use genco::{lang::rust::Tokens, quote};

use crate::Args;

pub fn generate_stream_trait_impl<'s>(
    args: &Args,
    timeout: u8,
    target: impl Into<Cow<'s, str>>,
    content_type: impl Into<Cow<'s, str>>,
    lightweight_type: &str,
) -> String {
    let trait_name = args.stream_message_trait_path();
    let target = target.into();
    let content_type = content_type.into();
    let size_const = &args.stream_max_payload_size_path();
    let stream_id_path = &args.stream_id_path();
    let stream_type_path = &args.stream_type_path();

    let tokens: Tokens = quote! {
        impl $trait_name for $target {
            const PREPARED: Self = Self::PREPARED;
            const TIMEOUT: u8 = $timeout;
            type Content = $content_type;
            type Lightweight = $lightweight_type;

            fn data(&self) -> &[u8; $size_const] {
                &self.data
            }

            fn data_mut(&mut self)  -> &mut [u8; $size_const] {
                &mut self.data
            }

            fn with_stream_id(&mut self, value: $stream_id_path) -> &mut Self {
                self.with_stream_id(value)
            }

            fn stream_id(&self) -> $stream_id_path {
                <$stream_id_path>::new((self.meta & 0b_1111_1110) >> 1)
            }

            fn with_discriminant(&mut self, value: $stream_type_path) -> &mut Self {
                self.with_discriminant(value)
            }

            fn discriminant(&self) -> $stream_type_path {
                unsafe {
                    let value = self.meta & 0b_0000_0001;
                    core::mem::transmute::<u8, $stream_type_path>(value)
                }
            }
        }
    };
    tokens.to_string().unwrap()
}
