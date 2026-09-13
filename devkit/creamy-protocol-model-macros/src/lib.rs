mod symbol;
mod token;

use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Token, attributes(element, token))]
pub fn derive_token(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    proc_macro::TokenStream::from_iter([
        token::generate_with_ident_impl(&ast),
        token::generate_vector_element_trait_impl(&ast),
    ])
}

#[proc_macro_derive(Symbol, attributes(key))]
pub fn derive_symbol(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    symbol::generate_symbol_trait_impl(&ast)
}
