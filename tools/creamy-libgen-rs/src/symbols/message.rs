use creamy_libgen::{
    EnrichedSingleMessageSymbol, SymbolIterator,
    proxy::{EnrichedFieldSymbol, EnrichedFieldType, SpecialFieldType},
};
use genco::{lang::rust::Tokens, quote};

use crate::{
    args::Args, derive::generate_derive, path_to_string, utils::generate_message_const_block,
};

pub fn generate_builder_pattern<'s, I>(
    args: &Args,
    symbol: &EnrichedSingleMessageSymbol<'s, I>,
) -> Tokens
where
    I: SymbolIterator<EnrichedFieldSymbol<'s>>,
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
                EnrichedFieldType::Array { kind, len } => format!("[{kind}; {len}]"),
                EnrichedFieldType::Special(s) => match s {
                    SpecialFieldType::SubscriberId => args.subscriber_id_path(),
                    SpecialFieldType::GroupId => args.group_id_path(),
                },
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
        generate_message_const_block(args, n, symbol.group, symbol.kind, symbol.dispatch_value);
    quote! {
        impl $n {
            $const_block

            $functions
        }
    }
}

pub fn generate_message_trait_impl(args: &Args, message: &str, group: u8, kind: u8) -> Tokens {
    let group_id = &args.group_id_path();
    let t = args.message_trait_path();
    quote! {
        impl $t for $message {
            const GROUP: $group_id = $group_id::new_u8($group).unwrap();
            const KIND: u8 = $kind;
        }
    }
}

pub fn generate_typed_message_trait_impl(args: &Args, message: &str) -> Tokens {
    let t = args.typed_message_trait_path();
    let sub_id = args.subscriber_id_path();
    let grp_id = args.group_id_path();
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

            fn group(&self) -> u8 {
                self.group.as_u8()
            }

            fn with_group(&mut self, value: $grp_id) -> &mut Self {
                self.group = value;
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

pub fn generate_single_message<'s, I>(
    args: &Args,
    symbol: &EnrichedSingleMessageSymbol<'s, I>,
) -> Tokens
where
    I: SymbolIterator<EnrichedFieldSymbol<'s>>,
{
    let name = symbol.name;
    let derive = generate_derive(args, &[]);
    let fields = symbol
        .fields
        .clone()
        .map(|f| {
            let n = f.name;
            let t = match f.kind {
                EnrichedFieldType::Type { name } => path_to_string(&name),
                EnrichedFieldType::Array { kind, len } => format!("[{kind}; {len}]"),
                EnrichedFieldType::Special(s) => match s {
                    SpecialFieldType::SubscriberId => args.subscriber_id_path(),
                    SpecialFieldType::GroupId => args.group_id_path(),
                },
            };
            quote! {
                pub $n: $t,
            }
        })
        .collect::<Vec<_>>();

    let typed_message_impl = generate_typed_message_trait_impl(args, name);
    let message_impl = generate_message_trait_impl(args, name, symbol.group, symbol.kind);
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
