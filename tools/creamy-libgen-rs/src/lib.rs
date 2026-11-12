#![allow(clippy::too_many_lines)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::missing_errors_doc)]

pub mod libgen {
    pub use creamy_libgen::prelude::*;
}

mod args;
mod binary_utils;
mod derive;
mod dispatcher;
pub mod script;
mod symbols;
mod utils;

use std::{
    borrow::Cow,
    fmt::{Display, Write as FmtWrite},
    io::Write as IoWrite,
};

pub use args::Args;
use creamy_libgen::prelude::*;
use genco::{lang::rust::Tokens, quote};
pub use libgen::Target;

const fn repr_to_string(repr: PrimitiveRepr) -> Cow<'static, str> {
    Cow::Borrowed(match repr {
        PrimitiveRepr::U8 => "u8",
        PrimitiveRepr::U16 => "u16",
        PrimitiveRepr::U32 => "u32",
        PrimitiveRepr::U64 => "u64",
        PrimitiveRepr::I8 => "i8",
        PrimitiveRepr::I16 => "i16",
        PrimitiveRepr::I32 => "i32",
        PrimitiveRepr::I64 => "i64",
    })
}

enum Access {
    Pub,
    Crate,
}
impl Display for Access {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Access::Pub => write!(f, "pub"),
            Access::Crate => write!(f, "pub(crate)"),
        }
    }
}

struct Module {
    access: Access,
    name: String,
    tokens: Vec<Tokens>,
}

impl Module {
    fn into_tokens(self) -> Tokens {
        let a = self.access.to_string();
        let n = self.name;
        let t = self.tokens;
        quote! {
            $a mod $n {
                $t
            }
        }
    }
}

pub struct RustGen<W: IoWrite> {
    args: Args,
    modules: Vec<Module>,
    writer: W,
}

impl<W: IoWrite> RustGen<W> {
    pub fn new(args: Args, mut writer: W) -> Self {
        let _ = writer.write("#[allow(unused)]\n".as_bytes());
        Self {
            args,
            modules: vec![],
            writer,
        }
    }

    fn push_tokens(&mut self, tokens: Tokens) {
        self.modules.last_mut().unwrap().tokens.push(tokens);
    }
}

impl<'s, W: IoWrite> CodeGenerator<'s> for RustGen<W> {
    fn start_protocol(&mut self, protocol: &'s str, id: Option<u8>) {
        let access = if self.modules.is_empty() {
            Access::Crate
        } else {
            Access::Pub
        };

        let id = if let Some(id) = id {
            quote! {
                pub const ID: u8 = $id;
            }
        } else {
            quote! {}
        };

        self.modules.push(Module {
            access,
            name: protocol.to_string(),
            tokens: vec![id],
        });
    }

    fn generate_single_message<I>(&mut self, symbol: EnrichedSingleMessageSymbol<'s, I>)
    where
        I: SymbolIterator<EnrichedFieldSymbol<'s>>,
    {
        self.push_tokens(symbols::generate_single_message(&self.args, &symbol));
    }

    fn generate_stream_message<F, C>(&mut self, symbol: EnrichedStreamMessageSymbol<'s, F, C>)
    where
        F: SymbolIterator<EnrichedFieldSymbol<'s>>,
        C: SymbolIterator<EnrichedStreamFieldSymbol<'s>>,
    {
        self.push_tokens(symbols::generate_stream_message::<F, C>(
            &symbol, &self.args,
        ));
    }

    fn generate_bitset<I>(&mut self, symbol: EnrichedBitsetSymbol<'s, I>)
    where
        I: SymbolIterator<EnrichedBitsetValueSymbol<'s>>,
    {
        self.push_tokens(symbols::generate_bitset(&self.args, symbol));
    }

    fn generate_flags<I>(&mut self, symbol: EnrichedFlagsSymbol<'s, I>)
    where
        I: Iterator<Item = &'s str>,
    {
        self.push_tokens(symbols::generate_flags(&self.args, symbol));
    }

    fn generate_enum<I>(&mut self, symbol: EnrichedEnumSymbol<'s, I>)
    where
        I: Iterator<Item = EnrichedVariantSymbol<'s>>,
    {
        self.push_tokens(symbols::generate_enum(&self.args, symbol));
    }

    fn generate_struct<I>(&mut self, symbol: EnrichedStructSymbol<'s, I>)
    where
        I: Iterator<Item = EnrichedFieldSymbol<'s>>,
    {
        self.push_tokens(symbols::generate_struct(&self.args, symbol));
    }

    fn end_protocol(&mut self) {
        if self.modules.len() <= 1 {
            return;
        }

        let protocol = self.modules.pop().unwrap().into_tokens();
        self.modules.last_mut().unwrap().tokens.push(protocol);
    }

    fn generate_dispatcher<P, M>(&mut self, protocols: P, messages: M)
    where
        P: Iterator<Item = ::creamy_libgen::Path>,
        M: Iterator<Item = ::creamy_libgen::Path>,
    {
        let tokens = dispatcher::generate_dispatcher(&self.args, protocols, messages);
        self.push_tokens(tokens);
    }

    fn generate_plugin_protocol_table(&mut self, pht: PerfectHashTable) {
        let mut t = String::with_capacity(2048);
        t.push('[');
        for id in &pht.table {
            let _ = write!(&mut t, "{id}u8, ");
        }
        t.push(']');

        let len = pht.table.len();
        self.push_tokens(quote! {
            #[unsafe(no_mangle)]
            #[doc(hidden)]
            static __INTERNAL__TABLE: [u8; $len] = $t;

            #[unsafe(no_mangle)]
            #[doc(hidden)]
            static __INTERNAL__TABLE_SIZE: u8 = $len;
        });

        let mut s = String::with_capacity(1024);
        s.push('[');
        for id in &pht.special_table {
            let _ = write!(&mut s, "{id}u64, ");
        }
        s.push(']');

        let len = pht.special_table.len();
        self.push_tokens(quote! {
            #[unsafe(no_mangle)]
            #[doc(hidden)]
            static __INTERNAL__SPECIAL: [u64; $len] = $s;

            #[unsafe(no_mangle)]
            #[doc(hidden)]
            static __INTERNAL__SPECIAL_TABLE_SIZE: u8 = $len;
        });
    }

    fn generate_host_protocol_table(&mut self, pht: PerfectHashTable) {
        let mut t = String::with_capacity(2048);
        t.push('[');
        for id in &pht.table {
            let _ = write!(&mut t, "{id}u8, ");
        }
        t.push(']');

        let len = pht.table.len();
        self.push_tokens(quote! {
            pub static TABLE: [u8; $len] = $t;
        });

        let mut s = String::with_capacity(1024);
        s.push('[');

        for id in &pht.special_table {
            let _ = write!(&mut s, "{id}u64, ");
        }

        s.push(']');

        let len = pht.special_table.len();
        self.push_tokens(quote! {
            pub static SPECIAL: [u64; $len] = $s;
        });
    }

    fn generate_host_package(&mut self, binary: Vec<u8>) {
        use std::fmt::Write;
        let len = binary.len();
        let mut content = String::new();
        for byte in binary {
            let _ = write!(&mut content, "{byte}u8, ");
        }

        self.push_tokens(quote! {
            pub static PACKAGE: &[u8; $len] = &[$content];
        });
    }

    fn flush(&mut self) -> GenResult {
        assert_eq!(self.modules.len(), 1);
        if let Some(global_module) = self.modules.pop() {
            let content = global_module.into_tokens().to_string().unwrap();
            self.writer.write_all(content.as_bytes())?;
            self.writer.flush()?;
            Ok(())
        } else {
            panic!("Missing protocol");
        }
    }
}

fn path_to_string(path: &Path) -> String {
    match path {
        Path::Global { name } => name.clone(),
        Path::Absolute { components } => {
            format!("crate::generated::{}", components.join("::"))
        }
    }
}
