use creamy_libgen::{
    SymbolIterator,
    proxy::{EnrichedStreamFieldSymbol, EnrichedStreamFieldType},
};
use genco::{lang::rust::Tokens, quote};

use crate::Args;

fn generate_trait_impl(args: &Args, target: &str, content: Tokens) -> Tokens {
    let trait_name = args.binary_writer_trait_path();
    let write_trait = args.binary_write_trait_path();
    let endian_trait = args.binary_endianness_trait_path();
    let error = args.binary_write_error_path();
    quote! {
        impl $trait_name for $target {
            fn write<W: $write_trait<Error: Send + Sync + 'static>, E: $endian_trait>(&self, writer: &mut W) -> Result<(), $error<W::Error>> {
                $content
            }
        }
    }
}

pub fn generate_binary_writer_trait_impl(args: &Args, target: &str) -> Tokens {
    let content = quote! {
        let slice = unsafe {
            core::mem::transmute::<&Self, &[u8; core::mem::size_of::<Self>()]>(self)
        };
        slice.write::<W, E>(writer)
    };

    generate_trait_impl(args, target, content)
}

fn generate_writer_body<'s, I>(fields: I) -> Option<Tokens>
where
    I: SymbolIterator<EnrichedStreamFieldSymbol<'s>>,
{
    let mut array_write = vec![];
    let mut tokens = vec![];
    let mut from = 0;
    let mut to = 0;

    for f in fields {
        let name = &f.name;
        match f.kind {
            EnrichedStreamFieldType::Type { .. } => {
                to += f.type_id.size().value() as usize;
            }
            EnrichedStreamFieldType::StaticArray { len, .. } => {
                to += f.type_id.size().value() as usize * len as usize;
            }
            EnrichedStreamFieldType::DynamicArray { kind } => {
                if to != from {
                    tokens.push(quote! {
                        //
                        let sub_slice = &slice[offset..$to + offset];
                        let sub_slice = unsafe {
                            &*(&sub_slice[0] as *const u8 as *const [u8; $to])
                        };
                        sub_slice.write::<W, E>(writer)?;
                        offset += size_of::<alloc::boxed::Box<[$kind]>>() + $to;
                    });
                }
                tokens.push(quote! {
                    (self.$name.len() as u32).write::<W, E>(writer)?;
                });
                array_write.push(quote! {
                    for byte in &self.$name {
                        byte.write::<W, E>(writer)?;
                    }
                });
                from = 0;
                to = 0;
            }
        }
    }

    if to != 0 {
        tokens.push(quote! {
            let sub_slice = &slice[offset..$to + offset];
            let sub_slice = unsafe {
                &*(&sub_slice[0] as *const u8 as *const [u8; $to])
            };
            sub_slice.write::<W, E>(writer)?;
        });
    }

    if tokens.is_empty() {
        None
    } else {
        Some(quote! {
            let mut offset = 0;
            $tokens

            $array_write
        })
    }
}

pub fn generate_binary_writer_trait_impl_with_partial_slices<'s, I>(
    args: &Args,
    target: &str,
    fields: I,
) -> Tokens
where
    I: SymbolIterator<EnrichedStreamFieldSymbol<'s>>,
{
    let body = match generate_writer_body(fields) {
        Some(tokens) => {
            quote! {
                let slice = unsafe {
                    core::mem::transmute::<&Self, &[u8; core::mem::size_of::<Self>()]>(self)
                };

                $tokens
            }
        }
        None => Tokens::default(),
    };

    generate_trait_impl(
        args,
        target,
        quote! {
            $body
            Ok(())
        },
    )
}
