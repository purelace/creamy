use std::{borrow::Cow, io::Write};

use creamy_libgen::{
    SymbolIterator,
    package::model::symbols::PrimitiveRepr,
    proxy::{EnrichedBitsetSymbol, EnrichedBitsetValueSymbol},
};
use genco::{lang::rust::Tokens, quote};

use crate::{
    Args, extend_derive,
    generator::{
        Access, Argument, Body, BodyLine, Field, Function, Impl, Pass, Struct, StructContent,
    },
    repr_to_string,
};

fn build_reset_mask(bytes: u8, bits: u8, shift: u8, backing_type: PrimitiveRepr) -> String {
    let bytes = bytes as usize;
    let bits = bits as usize;
    let shift = shift as usize;

    assert!(shift < bytes * 8);
    let mut bit = 0;
    let mut string = String::with_capacity(bytes * 8 + bytes + 2);
    string.push('0');
    string.push('b');

    let end_pos = bytes * 8 - shift;
    let start_pos = end_pos - bits;

    for _ in 0..start_pos {
        if bit != 0 && bit % 8 == 0 {
            string.push('_');
        }
        string.push('1');
        bit += 1;
    }

    for _ in 0..bits {
        if bit != 0 && bit % 8 == 0 {
            string.push('_');
        }
        string.push('0');
        bit += 1;
    }

    for _ in 0..(bytes * 8) - bit {
        if bit != 0 && bit % 8 == 0 {
            string.push('_');
        }
        string.push('1');
        bit += 1;
    }

    match backing_type {
        PrimitiveRepr::I8 => string.push_str("u8 as i8"),
        PrimitiveRepr::I16 => string.push_str("u16 as i16"),
        PrimitiveRepr::I32 => string.push_str("u32 as i32"),
        PrimitiveRepr::I64 => string.push_str("u64 as i64"),
        _ => {}
    }

    string
}

fn build_mask(bytes: u8, bits: u8) -> String {
    let bytes = bytes as usize;
    let bits = bits as usize;

    let mut bit = 0;
    let mut string = String::with_capacity(bytes * 8 + bytes + 2);
    string.push('0');
    string.push('b');
    for _ in 0..(bytes * 8 - bits) {
        if bit != 0 && bit % 8 == 0 {
            string.push('_');
        }
        string.push('0');
        bit += 1;
    }

    for _ in 0..bits {
        if bit % 8 == 0 {
            string.push('_');
        }
        string.push('1');
        bit += 1;
    }
    string
}

fn generate_get_value(var: &str, symbol: &EnrichedBitsetValueSymbol) -> Tokens {
    let start = symbol.start_pos;
    let end = symbol.end_pos;
    let backing_type = &repr_to_string(symbol.backing_type);

    let build = (start..end)
        .map(|i| quote! { self.data[$i], })
        .collect::<Vec<_>>();

    quote! { let $var = $backing_type::from_be_bytes([$build]); }
}

fn generate_read_body(symbol: EnrichedBitsetValueSymbol) -> Tokens {
    let mask = build_mask(symbol.read_window_bytes, symbol.bits);
    let repr = repr_to_string(symbol.repr);
    let cast = match symbol.repr {
        PrimitiveRepr::U8 | PrimitiveRepr::U16 | PrimitiveRepr::U32 | PrimitiveRepr::U64 => {
            let shift = symbol.shift;
            if shift == 0 {
                quote! {
                    (value & $mask) as $repr
                }
            } else {
                quote! {
                    ((value >> $shift) & $mask) as $repr
                }
            }
        }
        PrimitiveRepr::I8 | PrimitiveRepr::I16 | PrimitiveRepr::I32 | PrimitiveRepr::I64 => {
            let container_bits = symbol.read_window_bytes * 8;
            let left_shift = container_bits - symbol.bits - symbol.shift;
            let right_shift = container_bits - symbol.bits;

            if left_shift == 0 && right_shift != 0 {
                quote! { (value >> $right_shift) as $repr }
            } else if left_shift != 0 && right_shift == 0 {
                quote! { (value << $left_shift) as $repr }
            } else {
                quote! { ((value << $left_shift) >> $right_shift) as $repr }
            }
        }
    };

    let get_value = generate_get_value("value", &symbol);
    quote! {
        $get_value
        $cast
    }
}

fn generate_set_value(symbol: &EnrichedBitsetValueSymbol) -> Tokens {
    let start = symbol.start_pos;
    let end = symbol.end_pos;
    quote! {
        let bytes = result.to_be_bytes();
        self.data[$start..$end].copy_from_slice(&bytes);
    }
}

fn generate_set_body(symbol: EnrichedBitsetValueSymbol, add_debug_asserts: bool) -> Tokens {
    //TODO: добавить unchecked вариант
    let assert = if add_debug_asserts {
        let max_value = 2u64.pow(u32::from(symbol.bits));
        if symbol.repr.is_signed() {
            let min = -((max_value / 2).cast_signed());
            let max = (max_value / 2).cast_signed() - 1;

            quote! {
                debug_assert!(
                    value >= $min && value <= $max, "Value out of range"
                );
            }
        } else {
            quote! {
                debug_assert!(
                    value < $max_value, "Value out of range"
                );
            }
        }
    } else {
        quote!()
    };

    let reset_mask = build_reset_mask(
        symbol.read_window_bytes,
        symbol.bits,
        symbol.shift,
        symbol.backing_type,
    );
    let mask = build_mask(symbol.read_window_bytes, symbol.bits);

    let backing_type = &repr_to_string(symbol.backing_type);
    let shift = symbol.shift;
    let set_value = generate_set_value(&symbol);
    let value = if shift == 0 {
        quote! {
            let value = value as $backing_type & $mask;
        }
    } else {
        quote! {
            let value = (value as $backing_type & $mask) << $shift;
        }
    };
    let old_value = generate_get_value("old_value", &symbol);
    quote! {
        $assert
        $value
        $old_value
        let temp = old_value & $reset_mask;
        let result = temp | value;
        $set_value
        self
    }
}

pub fn generate_bitset<'s, I, W>(args: &Args, symbol: EnrichedBitsetSymbol<'s, I>) -> Struct<'s, W>
where
    I: SymbolIterator<EnrichedBitsetValueSymbol<'s>>,
    W: Write,
{
    let mut bitset = Struct::new(symbol.name);
    bitset.access = Access::Pub;
    bitset
        .derives
        .inner
        .push(args.binary_write_derive_path().into());
    bitset
        .derives
        .inner
        .push(args.binary_read_derive_path().into());
    extend_derive(&mut bitset.derives, args);

    let mut impl_block = Impl {
        target: symbol.name.into(),
        functions: vec![],
        custom: vec![],
    };

    let mut bits = 0;
    for symbol in symbol.values {
        bits += symbol.bits;

        let name = symbol.name;
        let repr = &repr_to_string(symbol.repr);
        let body = generate_read_body(symbol);
        let read_fn = quote! {
            pub fn $name(&self) -> $repr {
                $body
            }
        };

        let body = generate_set_body(symbol, args.debug_asserts);
        let set_fn = quote! {
            pub fn set_$name(&mut self, value: $repr) -> &mut Self{
                $body
            }
        };

        impl_block
            .custom
            .push(Box::new(read_fn.to_string().unwrap()));

        impl_block
            .custom
            .push(Box::new(set_fn.to_string().unwrap()));

        impl_block.functions.push(Function {
            access: Access::Pub,
            is_const: false,
            is_extern: false,
            name: Cow::Owned(format!("with_{}", symbol.name)),
            generics: vec![],
            self_pass: Some(Pass::MutMove),
            args: vec![Argument {
                name: "value".into(),
                kind: repr_to_string(symbol.repr),
                pass: Pass::Move,
            }],
            ret: Some(Cow::Borrowed("Self")),
            body: Body {
                lines: vec![
                    BodyLine {
                        content: format!("self.set_{}(value);", symbol.name).into(),
                        depth: 0,
                    },
                    BodyLine {
                        content: "self".into(),
                        depth: 0,
                    },
                ],
            },
            inline: false,
        });
    }

    bitset.impls.push(impl_block);

    let bytes = bits.div_ceil(8);
    bitset.content = StructContent::Fields(vec![Field {
        access: Access::Pub,
        name: Cow::Borrowed("data"),
        kind: Cow::Owned(format!("[u8; {bytes}]")),
        comment: None,
    }]);

    bitset
}
