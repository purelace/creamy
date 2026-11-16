mod content;
mod lightweight;
mod stream_message;

use creamy_libgen::{
    SymbolIterator,
    proxy::{EnrichedFieldSymbol, EnrichedMessageSymbol, MessageType},
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
    symbols::message::{
        generate_basic_field_list, generate_message_trait_impl, generate_typed_message_trait_impl,
    },
    utils::generate_message_const_block,
};

fn generate_stream_module<'s, F, C>(args: &Args, symbol: &EnrichedMessageSymbol<'s, F, C>) -> Tokens
where
    F: SymbolIterator<EnrichedFieldSymbol<'s>>,
    C: SymbolIterator<EnrichedFieldSymbol<'s>>,
{
    let MessageType::Stream { content_fields, .. } = &symbol.message_type else {
        unreachable!();
    };

    let n = symbol.name;
    let mn = symbol.name.to_snake_case();
    let content = generate_stream_content(args, n, content_fields.clone());
    let lightweight = generate_lightweight_content(args, n, content_fields.clone());
    let stream_trait = generate_stream_trait_impl(
        args,
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

pub fn generate_stream_message<'s, F, C>(
    symbol: &EnrichedMessageSymbol<'s, F, C>,
    args: &Args,
) -> Tokens
where
    F: SymbolIterator<EnrichedFieldSymbol<'s>>,
    C: SymbolIterator<EnrichedFieldSymbol<'s>>,
{
    let name = symbol.name;
    let fields = generate_basic_field_list::<F>(args, symbol.fields.clone());
    let derive = generate_derive(args, &[]);
    //TODO: iterate over fields
    let const_block = generate_message_const_block(
        args,
        name,
        symbol.protocol,
        symbol.kind,
        symbol.dispatch_value,
    );
    let kind_path = &args.stream_type_path();
    let id_path = &args.stream_id_path();
    let typed_message_impl = generate_typed_message_trait_impl(args, name);
    let message_impl = generate_message_trait_impl(args, name, symbol.protocol, symbol.kind);
    let module = generate_stream_module(args, symbol);
    quote! {
        $derive
        #[repr(C, align(32))]
        pub struct $name {
            $fields
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
