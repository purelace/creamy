use std::borrow::Cow;

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

pub enum SliceData<'s> {
    Partial { from: usize, to: usize },
    Array { name: Cow<'s, str> },
}

pub fn generate_binary_writer_trait_impl_with_partial_slices(
    args: &Args,
    target: &str,
    slices: Vec<SliceData<'_>>,
) -> Tokens {
    use core::fmt::Write;

    const SLICE: &str = "
    let slice = unsafe {
        core::mem::transmute::<&Self, &[u8; core::mem::size_of::<Self>()]>(self)
    };";

    let mut has_content = false;
    let mut content = String::new();
    let mut array_slices = String::new();

    for s in slices {
        match s {
            SliceData::Partial { from, to } => {
                //unsafe {
                //    &*(&self.data[1] as *const u8 as *const [u8; $size_const])
                //}
                has_content = true;
                let len = to - from;
                let _ = writeln!(
                    &mut content,
                    "
                    let sub_slice = &slice[{from}..{to}];
                    let sub_slice = unsafe {{
                        &*(&sub_slice[0] as *const u8 as *const [u8; {len}])
                    }};
                    sub_slice.write::<W, E>(writer)?;"
                );
            }
            SliceData::Array { name } => {
                let _ = writeln!(
                    &mut content,
                    "(self.{name}.len() as u32).write::<W, E>(writer)?;"
                );
                let _ = writeln!(
                    &mut array_slices,
                    "
                    for byte in &self.{name} {{
                        byte.write::<W, E>(writer)?;
                    }}
                "
                );
            }
        }
    }

    let slice = if has_content {
        quote!($SLICE)
    } else {
        Tokens::default()
    };

    generate_trait_impl(
        args,
        target,
        quote! {
            $slice
            $content
            $array_slices
            Ok(())
        },
    )
}
