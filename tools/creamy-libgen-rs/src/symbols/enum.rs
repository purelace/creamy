use creamy_libgen::{
    package::model::symbols::PrimitiveRepr,
    proxy::{EnrichedEnumSymbol, EnrichedVariantSymbol},
};
use genco::{lang::rust::Tokens, quote};

use crate::{args::Args, derive::generate_derive};

pub fn generate_enum<'s, I>(args: &Args, symbol: EnrichedEnumSymbol<'s, I>) -> Tokens
where
    I: Iterator<Item = EnrichedVariantSymbol<'s>>,
{
    let name = symbol.name;
    let derives = generate_derive(
        args,
        &[
            args.binary_write_derive_path(),
            args.binary_read_derive_path(),
        ],
    );
    let variants = symbol
        .variants
        .map(|v| {
            let n = v.name;
            let v = v.value.to_string();
            quote! { $n = $v, }
        })
        .collect::<Vec<_>>();

    let repr = match symbol.repr {
        PrimitiveRepr::U8 => "u8",
        PrimitiveRepr::U16 => "u16",
        PrimitiveRepr::U32 => "u32",
        PrimitiveRepr::U64 => "u64",
        PrimitiveRepr::I8 => "i8",
        PrimitiveRepr::I16 => "i16",
        PrimitiveRepr::I32 => "i32",
        PrimitiveRepr::I64 => "i64",
    };

    quote! {
        $derives
        #[repr($repr)]
        pub enum $name {
            $variants
        }
    }
}
