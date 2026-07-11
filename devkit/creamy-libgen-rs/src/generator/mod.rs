mod enumeration;
mod function;
mod r#impl;
mod module;
mod structure;
mod r#trait;
mod trait_impl;

use std::{borrow::Cow, fmt::Display};

pub use enumeration::{Enum, EnumVariant};
pub use function::{Argument, Body, BodyLine, Function};
pub use r#impl::Impl;
pub use module::Module;
pub use structure::{Field, Struct, StructContent};
pub use r#trait::{FunctionDefinition, Trait};
pub use trait_impl::{TraitImpl, TraitImplAssociatedType};

#[derive(Default, Debug, Copy, Clone)]
pub enum Access {
    #[default]
    None,
    Pub,
}

impl Display for Access {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Access::None => Ok(()),
            Access::Pub => write!(f, "pub"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Pass {
    MutMove,
    Move,
    Ref,
    Mut,
}

pub fn add_depth<W: std::io::Write>(writer: &mut W, depth: usize) -> Result<(), std::io::Error> {
    for _ in 0..depth {
        write!(writer, "    ")?;
    }
    Ok(())
}

#[derive(Default)]
pub struct DeriveList<'a> {
    pub inner: Vec<&'a str>,
}

impl DeriveList<'_> {
    #[must_use]
    pub const fn new() -> Self {
        Self { inner: vec![] }
    }

    fn write_to<W: std::io::Write>(
        &self,
        writer: &mut W,
        depth: usize,
    ) -> Result<(), std::io::Error> {
        if self.inner.is_empty() {
            return Ok(());
        }

        add_depth(writer, depth)?;
        write!(writer, "#[derive(")?;

        for (idx, derive) in self.inner.iter().enumerate() {
            write!(writer, "{derive}")?;
            if idx < self.inner.len() - 1 {
                write!(writer, ", ")?;
            }
        }

        writeln!(writer, ")]")
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Repr {
    None,
    Transparent,
    C { align: Option<u8> },
}

pub trait CodeBlock<W: std::io::Write> {
    fn write_to(&self, writer: &mut W, depth: usize) -> Result<(), std::io::Error>;
}

pub struct Const<'a> {
    pub access: Access,
    pub ident: Cow<'a, str>,
    pub kind: Cow<'a, str>,
    pub value: Cow<'a, str>,
}

impl<W: std::io::Write> CodeBlock<W> for Const<'_> {
    fn write_to(&self, writer: &mut W, depth: usize) -> Result<(), std::io::Error> {
        add_depth(writer, depth)?;
        writeln!(
            writer,
            "{} const {}: {} = {};",
            self.access, self.ident, self.kind, self.value
        )
    }
}
