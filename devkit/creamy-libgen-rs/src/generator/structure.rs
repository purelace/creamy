use std::borrow::Cow;

use super::{Access, Impl, TraitImpl};
use crate::generator::{CodeBlock, DeriveList, Repr, add_depth};

pub enum StructContent<'a> {
    Fields(Vec<Field<'a>>),
    Tuple(Vec<Cow<'a, str>>),
}

pub struct Struct<'a, W: std::io::Write> {
    pub repr: Repr,
    pub derives: DeriveList<'a>,
    pub access: Access,
    pub name: Cow<'a, str>,
    pub content: StructContent<'a>,
    pub trait_impls: Vec<TraitImpl<'a>>,
    pub impls: Vec<Impl<'a, W>>,
}

impl<'a, W: std::io::Write> Struct<'a, W> {
    #[must_use]
    pub fn new(name: impl Into<Cow<'a, str>>) -> Self {
        Self {
            repr: Repr::None,
            derives: DeriveList::new(),
            access: Access::None,
            name: name.into(),
            content: StructContent::Fields(vec![]),
            trait_impls: vec![],
            impls: vec![],
        }
    }

    pub fn write_to(&self, writer: &mut W, depth: usize) -> Result<(), std::io::Error> {
        match self.repr {
            Repr::None => {}
            Repr::Transparent => {
                add_depth(writer, depth)?;
                writeln!(writer, "#[repr(transparent)]")?;
            }
            Repr::C { align } => {
                add_depth(writer, depth)?;
                match align {
                    Some(value) => writeln!(writer, "#[repr(C, align({value}))]"),
                    None => writeln!(writer, "#[repr(C)]"),
                }?;
            }
        }

        self.derives.write_to(writer, depth)?;

        match &self.content {
            StructContent::Fields(fields) => {
                add_depth(writer, depth)?;
                writeln!(writer, "{} struct {} {{", self.access, self.name)?;
                for field in fields {
                    field.write_to(writer, depth + 1)?;
                }
                add_depth(writer, depth)?;
                writeln!(writer, "}}\n")?;
            }
            StructContent::Tuple(cows) => {
                add_depth(writer, depth)?;
                write!(writer, "{} struct {}(", self.access, self.name)?;
                for (idx, cow) in cows.iter().enumerate() {
                    if idx + 1 == cows.len() {
                        write!(writer, "{cow}")?;
                    } else {
                        write!(writer, "{cow}, ")?;
                    }
                }
                writeln!(writer, ");\n")?;
            }
        }

        for block in &self.trait_impls {
            block.write_to(writer, depth)?;
        }

        for block in &self.impls {
            block.write_to(writer, depth)?;
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct Field<'a> {
    pub access: Access,
    pub name: Cow<'a, str>,
    pub kind: Cow<'a, str>,
    pub comment: Option<Cow<'a, str>>,
}

impl Field<'_> {
    pub fn write_to<W: std::io::Write>(
        &self,
        writer: &mut W,
        depth: usize,
    ) -> Result<(), std::io::Error> {
        if let Some(comment) = self.comment.as_ref() {
            add_depth(writer, depth)?;
            writeln!(writer, "/* {comment} */")?;
        }

        add_depth(writer, depth)?;
        writeln!(writer, "{} {}: {},", self.access, self.name, self.kind)
    }
}
