#![allow(non_snake_case)]

use message_generator::{MessageGenerator, generate_struct};
use proc_macro::TokenStream;

pub fn define_message(input: TokenStream) -> TokenStream {
    let generator = MessageGenerator::default();
    let expanded = generate_struct(generator, input.into()).unwrap();
    TokenStream::from(expanded)
}
