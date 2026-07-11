use std::{borrow::Cow, io::Write};

use super::{Access, Argument, CodeBlock, Pass};
use crate::generator::add_depth;

pub struct FunctionDefinition<'s> {
    pub name: Cow<'s, str>,
    pub self_arg: Option<Pass>,
    pub args: Vec<Argument<'s>>,
    pub ret: Option<Cow<'s, str>>,
}

impl<W: Write> CodeBlock<W> for FunctionDefinition<'_> {
    fn write_to(&self, writer: &mut W, depth: usize) -> Result<(), std::io::Error> {
        add_depth(writer, depth)?;
        write!(writer, "fn {}(", self.name)?;

        if let Some(pass) = self.self_arg {
            let arg = match pass {
                Pass::MutMove => "mut self",
                Pass::Move => "self",
                Pass::Ref => "&self",
                Pass::Mut => "&mut self",
            };
            write!(writer, "{arg}")?;
        }

        if !self.args.is_empty() {
            write!(writer, ", ")?;
            for (idx, arg) in self.args.iter().enumerate() {
                arg.write_to(writer)?;
                if idx != self.args.len() - 1 {
                    write!(writer, ", ")?;
                }
            }
        }

        write!(writer, ")")?;

        if let Some(ret) = self.ret.as_ref() {
            write!(writer, " -> {ret}")?;
        }

        writeln!(writer, ";")
    }
}

impl<'s> FunctionDefinition<'s> {
    pub fn new(name: impl Into<Cow<'s, str>>) -> Self {
        Self {
            name: name.into(),
            self_arg: None,
            args: vec![],
            ret: None,
        }
    }

    pub const fn set_self(&mut self, value: Pass) -> &mut Self {
        self.self_arg = Some(value);
        self
    }

    pub fn add_argument(&mut self, argument: Argument<'s>) -> &mut Self {
        self.args.push(argument);
        self
    }

    //pub fn set_return(&mut self, value: impl Into<Cow<'s, str>>) -> &mut Self {
    //    self.ret = Some(value.into());
    //    self
    //}
}

pub struct TraitAssociatedType<'s> {
    pub name: Cow<'s, str>,
    pub bound: Cow<'s, str>,
}

impl<W: std::io::Write> CodeBlock<W> for TraitAssociatedType<'_> {
    fn write_to(&self, writer: &mut W, depth: usize) -> Result<(), std::io::Error> {
        add_depth(writer, depth)?;
        writeln!(writer, "type {}: {};", self.name, self.bound)
    }
}

pub struct Trait<'s> {
    pub access: Access,
    pub name: Cow<'s, str>,
    pub bound: Option<Cow<'s, str>>,
    pub types: Vec<TraitAssociatedType<'s>>,
    pub functions: Vec<FunctionDefinition<'s>>,
}

impl<'s> Trait<'s> {
    pub fn new(name: impl Into<Cow<'s, str>>) -> Self {
        Self {
            access: Access::Pub,
            name: name.into(),
            bound: None,
            types: vec![],
            functions: vec![],
        }
    }

    pub fn add_function(&mut self, function: FunctionDefinition<'s>) -> &mut Self {
        self.functions.push(function);
        self
    }
}

impl<W: Write> CodeBlock<W> for Trait<'_> {
    fn write_to(&self, writer: &mut W, depth: usize) -> Result<(), std::io::Error> {
        add_depth(writer, depth)?;
        if let Some(trait_bound) = &self.bound {
            writeln!(
                writer,
                "{} trait {}: {trait_bound} {{",
                self.access, self.name
            )?;
        } else {
            writeln!(writer, "{} trait {} {{", self.access, self.name)?;
        }

        for ty in &self.types {
            ty.write_to(writer, depth + 1)?;
        }

        for function in &self.functions {
            function.write_to(writer, depth + 1)?;
        }

        writeln!(writer, "}}")
    }
}
