use syn::{Attribute, Expr, Lit, Meta};

fn parse_magic_bytes(attr: &Attribute) -> syn::Result<Vec<u8>> {
    let Meta::List(meta_list) = &attr.meta else {
        return Err(syn::Error::new_spanned(
            attr,
            "Атрибут magic должен содержать аргументы, например: #[magic(0x50)]",
        ));
    };

    let expr: Expr = meta_list.parse_args()?;

    match expr {
        // Формат: Байтовый строковый литерал (b"WIN!" или br#"WIN!"#)
        Expr::Lit(syn::ExprLit {
            lit: syn::Lit::ByteStr(lit_byte_str),
            ..
        }) => {
            // .value() для байтовых строк возвращает Vec<u8>,
            // автоматически обрабатывая экранирование
            Ok(lit_byte_str.value())
        }

        // Формат 2: Одиночное число (например, 0x50 или 80)
        Expr::Lit(syn::ExprLit {
            lit: Lit::Int(lit_int),
            ..
        }) => {
            // Парсим число как u8
            let byte = lit_int.base10_digits().parse::<u8>().map_err(|_| {
                syn::Error::new_spanned(lit_int, "Число должно помещаться в тип u8 (0..255)")
            })?;
            Ok(vec![byte])
        }

        // Формат 3: Массив байтов в квадратных скобках [0x0, 0x1, 0x2]
        Expr::Array(expr_array) => {
            let mut bytes = Vec::new();

            for elem in &expr_array.elems {
                if let Expr::Lit(syn::ExprLit {
                    lit: Lit::Int(lit_int),
                    ..
                }) = elem
                {
                    let byte = lit_int.base10_digits().parse::<u8>().map_err(|_| {
                        syn::Error::new_spanned(lit_int, "Элемент массива должен быть валидным u8")
                    })?;
                    bytes.push(byte);
                } else {
                    return Err(syn::Error::new_spanned(
                        elem,
                        "Массив должен состоять только из целых чисел",
                    ));
                }
            }
            Ok(bytes)
        }

        _ => Err(syn::Error::new_spanned(
            meta_list,
            "Неподдерживаемый формат. Разрешены: строки, одиночные u8 или массивы [u8]",
        )),
    }
}

pub fn find_magic_bytes(attribute: &[Attribute]) -> Option<syn::Result<Vec<u8>>> {
    let attr = attribute.iter().find(|a| a.path().is_ident("magic"))?;
    Some(parse_magic_bytes(attr))
}
