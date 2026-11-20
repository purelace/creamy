use creamy_libgen::{
    SymbolIterator,
    prelude::SpecialType,
    proxy::{EnrichedFieldSymbol, EnrichedFieldType, EnrichedMessageSymbol},
};
use genco::{lang::rust::Tokens, quote};

use crate::{
    args::Args, derive::generate_derive, path_to_string, utils::generate_message_const_block,
};

pub fn generate_builder_pattern<'s, I, C>(
    args: &Args,
    symbol: &EnrichedMessageSymbol<'s, I, C>,
) -> Tokens
where
    I: SymbolIterator<EnrichedFieldSymbol<'s>>,
    C: SymbolIterator<EnrichedFieldSymbol<'s>>,
{
    let functions = symbol
        .fields
        .clone()
        .filter_map(|f| {
            if f.is_padding {
                return None;
            }

            let n = &f.name;
            let kind = &match f.kind {
                EnrichedFieldType::Type { name } => path_to_string(&name),
                EnrichedFieldType::StaticArray { kind, len } => format!("[{kind}; {len}]"),
                EnrichedFieldType::Special(s) => match s {
                    SpecialType::SubscriberId => args.subscriber_id_path(),
                    SpecialType::ProtocolId => args.protocol_id_path(),
                },
                EnrichedFieldType::HeapArray { .. } => {
                    unreachable!()
                }
            };

            Some(quote! {
                pub const fn set_$n(&mut self, value: $kind) -> &mut Self {
                    self.$n = value;
                    self
                }

                pub const fn with_$n(mut self, value: $kind) -> Self {
                    self.set_$n(value);
                    self
                }
            })
        })
        .collect::<Vec<_>>();

    let n = symbol.name;
    let const_block =
        generate_message_const_block(args, n, symbol.protocol, symbol.kind, symbol.dispatch_value);
    quote! {
        impl $n {
            $const_block

            $functions
        }
    }
}

pub fn generate_message_trait_impl(args: &Args, message: &str, protocol: u8, kind: u8) -> Tokens {
    let protocol_id = &args.protocol_id_path();
    let t = args.message_trait_path();
    quote! {
        impl $t for $message {
            const PROTOCOL: $protocol_id = $protocol_id::new_u8($protocol).unwrap();
            const KIND: u8 = $kind;
        }
    }
}

pub fn generate_typed_message_trait_impl(args: &Args, message: &str) -> Tokens {
    let t = args.typed_message_trait_path();
    let sub_id = args.subscriber_id_path();
    let protocol_id = args.protocol_id_path();
    quote! {
        impl $t for $message {
            fn dst(&self) -> u8 {
                self.dst.as_u8()
            }

            fn with_dst(&mut self, value: $sub_id) -> &mut Self {
                self.dst = value;
                self
            }

            fn src(&self) -> u8 {
                self.src.as_u8()
            }

            fn protocol(&self) -> u8 {
                self.protocol.as_u8()
            }

            fn with_protocol(&mut self, value: $protocol_id) -> &mut Self {
                self.protocol = value;
                self
            }

            fn kind(&self) -> u8 {
                self.kind
            }

            fn with_kind(&mut self, value: u8) -> &mut Self {
                self.kind = value;
                self
            }
        }
    }
}

pub fn generate_basic_field_list<'s>(
    args: &Args,
    fields: impl Iterator<Item = EnrichedFieldSymbol<'s>>,
) -> Vec<Tokens> {
    fields
        .map(|f| {
            let n = f.name;
            let t = match f.kind {
                EnrichedFieldType::Type { name } => path_to_string(&name),
                EnrichedFieldType::StaticArray { kind, len } => format!("[{kind}; {len}]"),
                EnrichedFieldType::Special(s) => match s {
                    SpecialType::SubscriberId => args.subscriber_id_path(),
                    SpecialType::ProtocolId => args.protocol_id_path(),
                },
                EnrichedFieldType::HeapArray { .. } => unreachable!(),
            };

            if f.is_padding {
                let t: Tokens = quote!($n: $t);
                let x = format!("\n/* {} */", t.to_string().unwrap());
                quote! {
                    $(x)
                }
            } else {
                quote! {
                    pub $n: $t,
                }
            }
        })
        .collect::<Vec<_>>()
}

pub fn generate_single_message<'s, I, C>(
    args: &Args,
    symbol: &EnrichedMessageSymbol<'s, I, C>,
) -> Tokens
where
    I: SymbolIterator<EnrichedFieldSymbol<'s>>,
    C: SymbolIterator<EnrichedFieldSymbol<'s>>,
{
    let name = symbol.name;
    let derive = generate_derive(args, &[]);
    let fields = generate_basic_field_list(args, symbol.fields.clone());

    let typed_message_impl = generate_typed_message_trait_impl(args, name);
    let message_impl = generate_message_trait_impl(args, name, symbol.protocol, symbol.kind);
    let builder_impl = generate_builder_pattern(args, symbol);

    quote! {
        $derive
        #[repr(C, align(32))]
        pub struct $name {
            $fields
        }

        $typed_message_impl

        $message_impl

        $builder_impl
    }
}
