use std::{borrow::Cow, io::Write};

use crate::generator::{CodeBlock, Function, add_depth};

pub struct Impl<'a, W: Write> {
    pub target: Cow<'a, str>,
    pub functions: Vec<Function<'a>>,
    pub custom: Vec<Box<dyn CodeBlock<W> + 'a>>,
}

impl<W: std::io::Write> CodeBlock<W> for Impl<'_, W> {
    fn write_to(&self, writer: &mut W, depth: usize) -> Result<(), std::io::Error> {
        add_depth(writer, depth)?;
        writeln!(writer, "impl {} {{", self.target)?;

        for block in &self.custom {
            block.write_to(writer, depth + 1)?;
        }

        for function in &self.functions {
            function.write_to(writer, depth + 1)?;
        }

        add_depth(writer, depth)?;
        writeln!(writer, "}}\n")
    }
}
