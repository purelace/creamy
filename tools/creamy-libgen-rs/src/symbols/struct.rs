use creamy_libgen::{
    EnrichedStructSymbol,
    proxy::{EnrichedFieldSymbol, EnrichedFieldType},
};
use genco::{lang::rust::Tokens, quote};

use crate::{
    args::Args, binary_utils::generate_binary_writer_trait_impl, derive::generate_derive,
    path_to_string,
};

pub fn generate_struct<'s, I>(args: &Args, symbol: EnrichedStructSymbol<'s, I>) -> Tokens
where
    I: Iterator<Item = EnrichedFieldSymbol<'s>>,
{
    let name = symbol.name;
    let derives = generate_derive(args, &[]);
    let fields = symbol
        .fields
        .map(|s| {
            let n = s.name;
            let t = match s.kind {
                EnrichedFieldType::Type { name } => path_to_string(&name),
                EnrichedFieldType::Array { kind, len } => format!("[{kind}; {len}]"),
            };
            quote! {
                pub $n: $t,
            }
        })
        .collect::<Vec<_>>();

    let binary_writer_impl = generate_binary_writer_trait_impl(args, symbol.name);

    quote! {
        $derives
        #[repr(C)]
        pub struct $name {
            $fields
        }

        $binary_writer_impl
    }
}
