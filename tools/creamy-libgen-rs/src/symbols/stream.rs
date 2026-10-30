mod content;
mod lightweight;
mod stream_message;

use creamy_libgen::{
    EnrichedStreamMessageSymbol, SymbolIterator,
    proxy::{EnrichedFieldSymbol, EnrichedStreamFieldSymbol},
};
use genco::{lang::rust::Tokens, quote};
use heck::ToSnakeCase;

use self::{
    content::generate_stream_content, lightweight::generate_lightweight_content,
    stream_message::generate_stream_trait_impl,
};
use crate::{
    Args,
    derive::generate_derive,
    symbols::message::{generate_message_trait_impl, generate_typed_message_trait_impl},
    utils::generate_message_const_block,
};

fn generate_stream_module<'s, F, C>(
    args: &Args,
    symbol: &EnrichedStreamMessageSymbol<'s, F, C>,
) -> Tokens
where
    F: SymbolIterator<EnrichedFieldSymbol<'s>>,
    C: SymbolIterator<EnrichedStreamFieldSymbol<'s>>,
{
    let n = symbol.name;
    let mn = symbol.name.to_snake_case();
    let content = generate_stream_content(args, n, symbol.content_fields.clone());
    let lightweight = generate_lightweight_content(args, n, symbol.content_fields.clone());
    let stream_trait = generate_stream_trait_impl(
        args,
        symbol.timeout,
        symbol.name,
        format!("{mn}::{n}Content"),
        &format!("{mn}::{n}Lightweight"),
    );
    quote! {
        $stream_trait

        pub mod $mn {
            extern crate alloc;

            $content

            $lightweight
        }
    }
}

fn message_header() -> Tokens {
    quote! {
        /* -------- HEADER -------- */
        pub dst: u8,
        pub group: u8,
        pub src: u8,
        pub kind: u8,
    }
}

pub fn generate_stream_message<'s, F, C>(
    symbol: &EnrichedStreamMessageSymbol<'s, F, C>,
    args: &Args,
) -> Tokens
where
    F: SymbolIterator<EnrichedFieldSymbol<'s>>,
    C: SymbolIterator<EnrichedStreamFieldSymbol<'s>>,
{
    let name = symbol.name;
    let derive = generate_derive(args, &[]);
    //TODO: iterate over fields
    let header = message_header();
    let const_block = generate_message_const_block(
        &args.message_size_path(),
        name,
        symbol.group,
        symbol.kind,
        symbol.dispatch_value,
    );
    let kind_path = &args.stream_type_path();
    let id_path = &args.stream_id_path();
    let typed_message_impl = generate_typed_message_trait_impl(args, name);
    let message_impl = generate_message_trait_impl(args, name, symbol.group, symbol.kind);
    let module = generate_stream_module(args, symbol);
    quote! {
        $derive
        #[repr(C, align(32))]
        pub struct $name {
            $header
            /* -------- STREAM -------- */
            pub meta: u8,
            pub data: [u8; 27],
        }

        $typed_message_impl

        $message_impl

        impl $name {
            $const_block

            pub const fn with_discriminant(&mut self, value: $kind_path) -> &mut Self {
                self.meta &= 0b_1111_1110;
                let value = unsafe {
                    core::mem::transmute::<$kind_path, u8>(value)
                };
                self.meta |= value;
                self
            }

            pub const fn with_stream_id(&mut self, value: $id_path) -> &mut Self {
                self.meta &= 0b_0000_0001;
                self.meta |= value.value() << 1;
                self
            }
        }

        $module
    }
}
