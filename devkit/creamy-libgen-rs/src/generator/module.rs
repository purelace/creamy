use std::borrow::Cow;

use crate::generator::{Access, CodeBlock, Enum, Struct, add_depth};

pub struct Module<'a, W: std::io::Write> {
    pub access: Access,
    pub name: Cow<'a, str>,
    pub enums: Vec<Enum<'a>>,
    pub structs: Vec<Struct<'a, W>>,
    pub modules: Vec<Module<'a, W>>,
    pub other: Vec<Box<dyn CodeBlock<W> + 'a>>,
}

impl<'a, W: std::io::Write> Module<'a, W> {
    #[must_use]
    pub fn new(name: impl Into<Cow<'a, str>>) -> Self {
        Self {
            access: Access::None,
            name: name.into(),
            enums: vec![],
            structs: vec![],
            modules: vec![],
            other: vec![],
        }
    }

    pub fn write_to(&self, writer: &mut W, depth: usize) -> Result<(), std::io::Error> {
        add_depth(writer, depth)?;
        writeln!(writer, "{} mod {} {{", self.access, self.name)?;

        for enum_ in &self.enums {
            enum_.write_to(writer, depth + 1)?;
        }

        for struct_ in &self.structs {
            struct_.write_to(writer, depth + 1)?;
        }

        for module in &self.modules {
            module.write_to(writer, depth + 1)?;
        }

        for other in &self.other {
            other.write_to(writer, depth + 1)?;
        }

        add_depth(writer, depth)?;
        writeln!(writer, "}}\n")
    }
}
