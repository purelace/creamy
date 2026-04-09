use macro_utils::{get_crate_root, types::MESSAGE_BUS_SUBSCRIBER_CRATE};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::protocol::resolved::{ChunkCoder, Field, PatternType};

#[derive(Debug, Clone)]
pub struct Message {
    pub name: Ident,
    pub ty: String,
    pub version: u8,
    pub fields: Vec<Field>,
    pub size: usize,
    pub chunk: Option<ChunkCoder>,
}

impl Message {
    pub fn name(&self) -> String {
        self.name.to_string()
    }

    pub fn get_mask(&self) -> String {
        let mut mask = vec![
            "0x00", // DST
            "0x00", // VERSION
            "0xFF", // TYPE (2 bytes),
            "0xFF",
        ];

        //for field in &self.fields {
        //    let byte = if field.maskable { "0xFF" } else { "0x00" };
        //    mask.extend(std::iter::repeat_n(byte, field.ty.get_size()));
        //}

        let mut array = String::new();
        array.push('[');
        for byte in mask {
            array.push_str(byte);
            array.push(',');
        }
        array.replace_range(array.len() - 1..array.len(), "]");
        array
    }

    pub fn generate_code(&self) -> TokenStream {
        let name = &self.name;
        let ty = &self.ty;
        let version = self.version;
        let fields = &self.fields;
        let coder = self.generate_chunk_coder();

        quote! {
            #[repr(C, align(32))]
            #[derive(Metadata, Default, Debug, Copy, Clone, PartialEq)]
            #[metadata(name = #ty, version = #version)]
            pub struct #name {
                pub dst: Destination,
                pub src: u8,
                pub kind: MessageType,
                pub version: u8,
                #(
                    pub #fields,
                )*
            }

            unsafe impl bytemuck::Zeroable for #name {}
            unsafe impl bytemuck::Pod for #name {}

            #coder
        }
    }

    fn generate_chunk_coder(&self) -> TokenStream {
        let Some(coder) = self.chunk.as_ref() else {
            return TokenStream::default();
        };

        let crate_root = get_crate_root(MESSAGE_BUS_SUBSCRIBER_CRATE);
        let chunk_struct_name = format_ident!("{}Chunk", self.name);
        let chunk_struct_fields = coder.patterns.iter().map(|f| {
            let name = &f.name;
            let ty = &f.ty;
            quote! {
                #name: #ty
            }
        });

        let chunk_name = &coder.name;
        let iterator = format_ident!("{}Iterator", &coder.name);

        let types = coder.patterns.iter().map(|p| &p.ty).collect::<Vec<_>>();
        let doc_lines = coder
            .patterns
            .iter()
            .map(|p| &p.name)
            .zip(types.iter())
            .map(|(n, t)| format!("* {n}: {t}"));

        let decode = coder.patterns.iter().map(|p| {
            let size = p.size;
            let name = &p.name;

            match &p.ty {
                PatternType::RunTime(_, src) => {
                    let src = format_ident!("{src}");
                    quote! {
                        assert_ne!(#src, 0, "Invalid message length");
                        let #name = String::from_utf8_lossy(&self.bytes[self.offset..self.offset + #src as usize]).to_string();
                        self.offset += #src as usize;
                    }
                },
                PatternType::CompileTime(handle) => {
                    quote! {
                        let #name =
                            #handle::from_le_bytes(self.bytes[self.offset..=self.offset].try_into().unwrap());
                        self.offset += #size;
                    }
                },
            }
        });

        let ret_names = coder.patterns.iter().map(|p| &p.name);
        let ret = quote! {
            Some(#chunk_struct_name {
                #(#ret_names,)*
            })
        };

        quote! {
            #[doc = "Chunk header: "]
            #[doc = "* count: u32"]
            #[doc = ""]
            #[doc = "Chunk payload:"]
            #(
                #[doc = #doc_lines]
            )*
            pub struct #chunk_name<'a> {
                bytes: &'a [u8],
            }

            #[doc = "Chunk payload"]
            #[derive(Debug)]
            pub struct #chunk_struct_name {
                #(
                    pub #chunk_struct_fields,
                )*
            }

            impl<'a> #chunk_name<'a> {
                #[must_use]
                #[inline]
                pub const fn new(bytes: &'a [u8]) -> Self {
                    Self { bytes }
                }

                fn into_iter(self) -> #iterator<'a> {
                    #iterator::new(self.bytes)
                }
            }

            pub struct #iterator<'a> {
                bytes: &'a [u8],
                offset: usize,
                count: u32,
                index: u32,
            }

            impl<'a> #iterator<'a> {
                fn new(bytes: &'a [u8]) -> Self {
                    let count = u32::from_le_bytes(bytes[..#crate_root::LENGTH_SIZE].try_into().unwrap());
                    Self {
                        bytes,
                        offset: #crate_root::LENGTH_SIZE,
                        count,
                        index: 0,
                    }
                }
            }

            impl Iterator for #iterator<'_> {
                type Item = #chunk_struct_name;

                fn next(&mut self) -> Option<Self::Item> {
                    if self.index >= self.count {
                        return None;
                    }

                    #(#decode)*

                    #ret
                }
            }

            impl<'a> IntoIterator for #chunk_name<'a> {
                type Item = #chunk_struct_name;

                type IntoIter = #iterator<'a>;

                fn into_iter(self) -> Self::IntoIter {
                    self.into_iter()
                }
            }
        }
    }
}
