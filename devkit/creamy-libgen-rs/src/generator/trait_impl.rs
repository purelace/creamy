use std::borrow::Cow;

use super::{Function, add_depth};
use crate::generator::CodeBlock;

pub struct AssociatedType<'a> {
    pub name: &'a str,
    pub kind: &'a str,
}

impl<W: std::io::Write> CodeBlock<W> for AssociatedType<'_> {
    fn write_to(&self, writer: &mut W, depth: usize) -> Result<(), std::io::Error> {
        add_depth(writer, depth)?;
        writeln!(writer, "type {} = {};", self.name, self.kind)
    }
}

pub struct TraitImpl<'a> {
    pub trait_name: Cow<'a, str>,
    pub target: &'a str,
    pub associated_types: Vec<AssociatedType<'a>>,
    pub functions: Vec<Function<'a>>,
}

impl TraitImpl<'_> {
    pub fn write_to<W: std::io::Write>(
        &self,
        writer: &mut W,
        depth: usize,
    ) -> Result<(), std::io::Error> {
        add_depth(writer, depth)?;
        writeln!(writer, "impl {} for {} {{", self.trait_name, self.target)?;

        for ty in &self.associated_types {
            ty.write_to(writer, depth + 1)?;
        }

        for function in &self.functions {
            function.write_to(writer, depth + 1)?;
        }

        add_depth(writer, depth)?;
        writeln!(writer, "}}\n")
    }
}
