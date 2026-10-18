mod symbol;
mod token;

use syn::{Data, DeriveInput, parse_macro_input};

#[proc_macro_derive(Token, attributes(element, token))]
pub fn derive_token(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let fields = match &ast.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            syn::Fields::Named(fields_named) => &fields_named.named,
            _ => {
                return syn::Error::new(
                    name.span(),
                    "Поддерживаются только структуры с именованными полями",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new(
                name.span(),
                "Макрос можно использовать только со структурами",
            )
            .to_compile_error()
            .into();
        }
    };

    proc_macro::TokenStream::from_iter([
        token::generate_with_ident_impl(&ast),
        token::generate_vector_element_trait_impl(&ast),
        token::generate_impl_block(name, fields),
    ])
}

#[proc_macro_derive(Symbol, attributes(key))]
pub fn derive_symbol(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    symbol::generate_symbol_trait_impl(&ast)
}
