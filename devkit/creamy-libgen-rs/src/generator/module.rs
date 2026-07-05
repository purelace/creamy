use crate::generator::{Access, Enum, Struct, add_depth};

pub struct Module<'a, W: std::io::Write> {
    pub access: Access,
    pub name: &'a str,
    pub enums: Vec<Enum<'a>>,
    pub structs: Vec<Struct<'a, W>>,
    pub modules: Vec<Module<'a, W>>,
}

impl<'a, W: std::io::Write> Module<'a, W> {
    #[must_use]
    pub const fn new(name: &'a str) -> Self {
        Self {
            access: Access::None,
            name,
            enums: vec![],
            structs: vec![],
            modules: vec![],
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

        add_depth(writer, depth)?;
        writeln!(writer, "}}\n")
    }
}
