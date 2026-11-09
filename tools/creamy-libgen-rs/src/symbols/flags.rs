use creamy_libgen::proxy::{EnrichedFlagsSymbol, FlagUnderlyingType};
use genco::{lang::rust::Tokens, quote};

use crate::{args::Args, derive::generate_derive};

fn generate_set_methods(target: &str) -> Tokens {
    quote! {
        impl $target {
            pub const fn set(&mut self, rhs: Self) {
                self.0 |= rhs.0;
            }

            pub const fn unset(&mut self, rhs: Self) {
                self.0 &= !rhs.0;
            }
        }
    }
}

fn generate_bitxor_impl(target: &str, kind: &str) -> Tokens {
    quote! {
        impl core::ops::BitXor for $target {
            type Output = $kind;

            fn bitxor(self, rhs: Self) -> Self::Output {
                $kind::bitxor(self.0, rhs.0)
            }
        }
    }
}

fn generate_bitor_impl(target: &str, kind: &str) -> Tokens {
    quote! {
        impl core::ops::BitOr for $target {
            type Output = $kind;

            fn bitor(self, rhs: Self) -> Self::Output {
                $kind::bitor(self.0, rhs.0)
            }
        }
    }
}

fn generate_iterator_impl(target: &str, repr: &str, max_iterations: usize) -> Tokens {
    quote! {
        impl IntoIterator for $target {
            type Item = Self;
            type IntoIter = $(target)Iterator;

            fn into_iter(self) -> Self::IntoIter {
                $(target)Iterator {
                    flags: self,
                    index: 1,
                }
            }
        }

        pub struct $(target)Iterator {
            flags: $target,
            index: $repr,
        }

        impl Iterator for $(target)Iterator {
            type Item = $target;

            fn next(&mut self) -> Option<Self::Item> {
                const MAX_ITERATIONS: $repr = $max_iterations;

                let mask = 1 << self.index;
                self.index += 1;

                if self.index >= MAX_ITERATIONS || self.flags.0 & mask == 0 {
                    return None;
                }

                Some($target(mask))
            }
        }
    }
}

#[allow(clippy::similar_names)]
pub fn generate_flags<'s, I>(args: &Args, symbol: EnrichedFlagsSymbol<'s, I>) -> Tokens
where
    I: Iterator<Item = &'s str>,
{
    let name = symbol.name;
    let repr = match symbol.underlying_type {
        FlagUnderlyingType::U8 => "u8",
        FlagUnderlyingType::U16 => "u16",
        FlagUnderlyingType::U32 => "u32",
        FlagUnderlyingType::U64 => "u64",
        FlagUnderlyingType::U128 => "u128",
    };
    let derives = generate_derive(
        args,
        &[
            "Default".to_string(),
            args.binary_write_derive_path(),
            args.binary_read_derive_path(),
        ],
    );

    let variants = symbol
        .options
        .enumerate()
        .map(|(i, s)| {
            let shift = i + 1;
            quote! {
                pub const $s: Self = Self(1 << $shift);
            }
        })
        .collect::<Vec<_>>();

    let set_methods = generate_set_methods(name);
    let bitor = generate_bitor_impl(name, repr);
    let bitxor = generate_bitxor_impl(name, repr);
    let iterator = generate_iterator_impl(name, repr, variants.len());

    quote! {
        $derives
        #[repr(transparent)]
        pub struct $name($repr);
        impl $name {
            $variants
        }
        $set_methods
        $bitor
        $bitxor
        $iterator
    }
}
