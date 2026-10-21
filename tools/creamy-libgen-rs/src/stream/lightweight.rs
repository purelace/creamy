use creamy_libgen::{
    SymbolIterator,
    proxy::{EnrichedStreamFieldSymbol, EnrichedStreamFieldType},
};
use genco::{lang::rust::Tokens, quote};

use crate::{Args, path_to_string};

pub fn generate_lightweight_content<'s, I>(args: &Args, target: &str, iter: I) -> Tokens
where
    I: SymbolIterator<EnrichedStreamFieldSymbol<'s>>,
{
    let content_target = &format!("{target}Content");
    let new_target = &format!("{target}Lightweight");
    let fields = iter
        .clone()
        .map(|f| {
            let name = f.name;
            let mut is_pub = !f.is_padding;
            let kind = match f.kind {
                EnrichedStreamFieldType::Type { name } => path_to_string(&name),
                EnrichedStreamFieldType::StaticArray { kind, len } => {
                    format!("[{kind}; {len}]")
                }
                EnrichedStreamFieldType::DynamicArray { .. } => {
                    is_pub = false;
                    args.array_path()
                }
            };
            let pub_ = if is_pub { quote!(pub) } else { quote!() };
            quote! {
                $pub_ $name: $kind,
            }
        })
        .collect::<Vec<_>>();

    let functions = iter
        .clone()
        .filter_map(|f| {
            let EnrichedStreamFieldType::DynamicArray { kind } = f.kind else {
                return None;
            };

            let name = &f.name;
            let kind = &kind;
            //let read = format!("read_{name}");

            //TODO: calculate offsets
            if kind == "u8" {
                Some(quote! {
                    pub fn read_$name(&self) -> &[u8] {
                        let len = self.$name.count() as usize;
                        unsafe {
                            let ptr = core::ptr::from_ref(self) as *const u8;
                            let additional = ptr.add(core::mem::size_of::<Self>());
                            core::slice::from_raw_parts(additional, len)
                        }
                    }
                })
            } else {
                todo!()
                //TODO: make const
                //Some(quote! {
                //    pub fn read_$name(&self) -> &[$kind] {
                //        let len = self.$name.count() as usize;
                //
                //        let new_len = len / core::mem::size_of::<$kind>();
                //        unsafe {
                //            let ptr = core::ptr::from_ref(self) as *const u8;
                //            let additional = ptr.add(core::mem::size_of::<Self>());
                //            let slice = core::slice::from_raw_parts(additional as *const $kind, new_len);
                //        }
                //        let slice = &self.data[..len];
                //        unsafe {
                //            let ptr = slice.as_ptr();
                //            core::mem::transmute::<&[u8], _>(slice);
                //            core::slice::from_raw_parts(ptr as *const $kind, new_len)
                //        }
                //    }
                //})
            }
        })
        .collect::<Vec<_>>();

    let target_fields = iter
        .map(|f| {
            let name = &f.name;
            let value = match f.kind {
                EnrichedStreamFieldType::Type { .. }
                | EnrichedStreamFieldType::StaticArray { .. } => {
                    quote!(self.$name)
                }
                EnrichedStreamFieldType::DynamicArray { .. } => {
                    quote!(alloc::boxed::Box::from(self.read_$name()))
                }
            };
            quote! {
                $name: $value,
            }
        })
        .collect::<Vec<_>>();

    let trait_path = args.stream_lightweight_content_path();

    quote! {
        #[repr(C)]
        pub struct $new_target {
            $fields
        }

        impl $new_target {
            //const fn additional_data(&self) -> &[u8] {
            //
            //}

            $functions

            pub fn to_original(&self) -> $content_target {
                $content_target {
                    $target_fields
                }
            }
        }

        impl $trait_path for $new_target {
            fn new(data: &mut [u8]) -> &Self {
                const SIZE: usize = core::mem::size_of::<$new_target>();

                unsafe {
                    let main_slice = &mut *(&mut data[0] as *mut u8 as *mut [u8; SIZE]);
                    core::mem::transmute::<&mut [u8; SIZE], &mut Self>(main_slice)
                }
            }
        }
    }
}
