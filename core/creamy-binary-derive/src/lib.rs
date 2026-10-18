mod attrs;
mod reader;
mod utility;
mod writer;

use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(BinRead, attributes(magic))]
pub fn derive_reader(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    reader::generate_reader_impl(&ast)
}

#[proc_macro_derive(BinWrite, attributes(magic))]
pub fn derive_writer(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    writer::generate_writer_impl(&ast)
}
