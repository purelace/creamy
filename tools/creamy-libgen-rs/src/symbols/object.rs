use creamy_libgen::proxy::{
    EnrichedFieldSymbol, EnrichedFieldType, EnrichedObjectSymbol, EnrichedStructSymbol,
};
use genco::{lang::rust::Tokens, quote};

use crate::{
    args::Args,
    binary_utils::{generate_binary_reader_trait_impl, generate_binary_writer_trait_impl},
    derive::generate_derive,
    path_to_string,
};

pub fn generate_object<'s, I>(args: &Args, symbol: EnrichedObjectSymbol<'s, I>) -> Tokens
where
    I: Iterator<Item = EnrichedFieldSymbol<'s>>,
{
    let name = symbol.name;
    let derives = generate_derive(args, &[]);
    let mut add_lifetime = false;
    let fields = symbol
        .fields
        .map(|s| {
            let n = s.name;

            let t = match s.kind {
                EnrichedFieldType::Type { name } => path_to_string(&name),
                EnrichedFieldType::StaticArray { kind, len } => format!("[{kind}; {len}]"),
                EnrichedFieldType::HeapArray { kind } => {
                    add_lifetime = true;
                    format!("&'a [{kind}]")
                }
                EnrichedFieldType::Special(_) => unreachable!(),
            };
            quote! {
                pub $n: $t,
            }
        })
        .collect::<Vec<_>>();

    //let binary_writer_impl = generate_binary_writer_trait_impl(args, symbol.name);
    //let binary_reader_impl = generate_binary_reader_trait_impl(args, symbol.name);
    quote! {
        $derives
        #[repr(C)]
        pub struct $name$(if add_lifetime {<'a>}) {
            $fields
        }

        //$binary_writer_impl

        //$binary_reader_impl

    }
}
