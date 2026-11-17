use creamy_libgen::proxy::{EnrichedLayoutEntry, EnrichedSerializationLayout};
use genco::{lang::rust::Tokens, quote};

use crate::Args;

fn generate_read_trait_impl(args: &Args, target: &str, content: Tokens) -> Tokens {
    let trait_name = args.binary_reader_trait_path();
    let read_trait = args.binary_read_trait_path();
    let endian_trait = args.binary_endianness_trait_path();
    let error = args.binary_read_error_path();
    quote! {
        impl $trait_name for $target {
            fn read<R: $read_trait, E: $endian_trait>(reader: &mut R) -> Result<Self, $error<R::Error>> {
                $content
            }
        }
    }
}

pub fn generate_binary_reader_trait_impl(args: &Args, target: &str) -> Tokens {
    let content = quote! {
        let bytes = <[u8; core::mem::size_of::<Self>()]>::read::<R, E>(reader)?;

        Ok(unsafe {
            core::mem::transmute::<[u8; core::mem::size_of::<Self>()], Self>(bytes)
        })
    };

    generate_read_trait_impl(args, target, content)
}

fn generate_write_trait_impl(args: &Args, target: &str, content: Tokens) -> Tokens {
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

    generate_write_trait_impl(args, target, content)
}

fn generate_writer_body(layout: EnrichedSerializationLayout) -> Option<Tokens> {
    let mut tokens = vec![];

    for entry in layout.items {
        match entry {
            EnrichedLayoutEntry::ArrayLength { name } => {
                tokens.push(quote! {
                    (self.$name.len() as u32).write::<W, E>(writer)?;
                });
            }
            EnrichedLayoutEntry::ArrayWrite { name } => {
                tokens.push(quote! {
                    for byte in &self.$name {
                        byte.write::<W, E>(writer)?;
                    }
                });
            }
            EnrichedLayoutEntry::Slice { to, kind } => {
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
            EnrichedLayoutEntry::LastSlice { to } => tokens.push(quote! {
                let sub_slice = &slice[offset..$to + offset];
                let sub_slice = unsafe {
                    &*(&sub_slice[0] as *const u8 as *const [u8; $to])
                };
                sub_slice.write::<W, E>(writer)?;
            }),
        }
    }

    if tokens.is_empty() {
        None
    } else {
        Some(quote! {
            let mut offset = 0;
            $tokens
        })
    }
}

pub fn generate_binary_writer_trait_impl_with_partial_slices(
    args: &Args,
    target: &str,
    layout: EnrichedSerializationLayout,
) -> Tokens {
    let body = match generate_writer_body(layout) {
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

    generate_write_trait_impl(
        args,
        target,
        quote! {
            $body
            Ok(())
        },
    )
}
