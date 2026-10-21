use creamy_libgen::{
    SymbolIterator,
    proxy::{EnrichedStreamFieldSymbol, EnrichedStreamFieldType},
};
use genco::{lang::rust::Tokens, quote};

use crate::{
    Args,
    binary_utils::{
        SliceData, generate_binary_writer_trait_impl,
        generate_binary_writer_trait_impl_with_partial_slices,
    },
    path_to_string,
};

pub fn generate_stream_content<'s, I>(args: &Args, message: &str, fields_iter: I) -> Tokens
where
    I: SymbolIterator<EnrichedStreamFieldSymbol<'s>>,
{
    let name = &format!("{message}Content");

    let mut additional_size_content = Vec::new();
    let mut slices = vec![];
    let mut from = 0;
    let mut to = 0;

    let fields = fields_iter
        .clone()
        .map(|f| {
            let name = &f.name;
            let kind = match f.kind {
                EnrichedStreamFieldType::Type { name } => {
                    to += f.metadata.size;
                    path_to_string(&name)
                }
                EnrichedStreamFieldType::StaticArray { kind, len } => {
                    to += f.metadata.size;
                    format!("[{kind}; {len}]")
                }
                EnrichedStreamFieldType::DynamicArray { kind } => {
                    //TODO: сделать проверку на переполнение
                    additional_size_content.push(format!("self.{}.len() as u32", f.name));

                    if to != from {
                        slices.push(SliceData::Partial { from, to });
                    }
                    slices.push(SliceData::Array {
                        name: f.name.clone(),
                    });
                    from += 8; // Size of array
                    to = from;

                    format!("alloc::boxed::Box<[{kind}]>")
                }
            };
            let pub_ = if f.is_padding { quote!() } else { quote!(pub) };
            quote! {
                $pub_ $name: $kind,
            }
        })
        .collect::<Vec<_>>();

    let trait_impl = if from == 0 && to != 0 {
        generate_binary_writer_trait_impl(args, name)
    } else {
        generate_binary_writer_trait_impl_with_partial_slices(args, name, slices)
    };

    let mut additional_data = additional_size_content.join("+").clone();
    if additional_data.is_empty() {
        additional_data = String::from("0");
    }

    let content_trait_path = args.stream_content_trait();

    let mut params = Vec::new();
    let args = fields_iter
        .filter_map(|f| {
            let name = &f.name;

            if f.is_padding {
                params.push(quote! {
                    $name: Default::default(),
                });
                return None;
            }

            let kind = match f.kind {
                EnrichedStreamFieldType::Type { name } => path_to_string(&name),
                EnrichedStreamFieldType::StaticArray { kind, len } => {
                    format!("[{kind}; {len}]")
                }
                EnrichedStreamFieldType::DynamicArray { kind } => {
                    format!("alloc::boxed::Box<[{kind}]>")
                }
            };

            params.push(quote!($name,));
            Some(quote! {
                $name: $kind,
            })
        })
        .collect::<Vec<_>>();

    quote! {
        #[repr(C)]
        pub struct $name {
            $fields
        }

        impl $name {
            //TODO: const
            pub fn new($args) -> Self {
                Self {
                    $params
                }
            }
        }

        $trait_impl

        impl $content_trait_path for $name {
            type Message = super::$message;

            fn additional_data(&self) -> u32 {
                $additional_data
            }
        }
    }
}
