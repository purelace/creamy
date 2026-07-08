use creamy_xmlc::{VariantValue, model::symbols::PrimitiveRepr};

use crate::generator::{Access, DeriveList, add_depth};

pub struct EnumVariant<'a> {
    pub name: &'a str,
    pub value: VariantValue,
}

impl EnumVariant<'_> {
    fn write_to<W: std::io::Write>(
        &self,
        writer: &mut W,
        depth: usize,
    ) -> Result<(), std::io::Error> {
        add_depth(writer, depth)?;
        writeln!(writer, "{} = {},", self.name, self.value)
    }
}

pub struct Enum<'a> {
    pub derives: DeriveList<'a>,
    pub access: Access,
    pub repr: PrimitiveRepr,
    pub name: &'a str,
    pub variants: Vec<EnumVariant<'a>>,
}

impl<'a> Enum<'a> {
    #[must_use]
    pub const fn new(name: &'a str) -> Self {
        Self {
            derives: DeriveList::new(),
            access: Access::None,
            repr: PrimitiveRepr::U8,
            name,
            variants: vec![],
        }
    }

    pub fn write_to<W: std::io::Write>(
        &self,
        writer: &mut W,
        depth: usize,
    ) -> Result<(), std::io::Error> {
        self.derives.write_to(writer, depth)?;

        add_depth(writer, depth)?;
        writeln!(writer, "#[repr({})]", self.repr)?;

        add_depth(writer, depth)?;
        writeln!(writer, "{} enum {} {{", self.access, self.name)?;

        for variant in &self.variants {
            variant.write_to(writer, depth + 1)?;
        }

        add_depth(writer, depth)?;
        writeln!(writer, "}}\n")
    }
}
