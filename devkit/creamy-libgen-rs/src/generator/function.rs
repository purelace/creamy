use std::{borrow::Cow, io::Write};

use super::{Access, CodeBlock, Pass, add_depth};

//TODO: remove Default
#[derive(Default)]
pub struct Function<'a> {
    pub access: Access,
    pub is_const: bool,
    pub name: Cow<'a, str>,
    pub self_pass: Option<Pass>,
    pub args: Vec<Argument<'a>>,
    pub ret: Option<Cow<'a, str>>,
    pub body: Body<'a>,
    pub inline: bool,
}

impl<'a, W: Write + 'a> CodeBlock<W> for Function<'a> {
    fn write_to(&self, writer: &mut W, depth: usize) -> Result<(), std::io::Error> {
        add_depth(writer, depth)?;
        if self.inline {
            writeln!(writer, "#[inline(always)]")?;
        }

        add_depth(writer, depth)?;
        write!(writer, "{}", self.access)?;
        if self.is_const {
            write!(writer, " const ")?;
        }
        write!(writer, " fn {}(", self.name)?;

        if let Some(pass) = self.self_pass {
            let arg = match pass {
                Pass::MutMove => "mut self",
                Pass::Move => "self",
                Pass::Ref => "&self",
                Pass::Mut => "&mut self",
            };
            write!(writer, "{arg}")?;
        }

        if !self.args.is_empty() {
            if self.self_pass.is_some() {
                write!(writer, ", ")?;
            }

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

        writeln!(writer, " {{")?;

        self.body.write_to(writer, depth + 1)?;

        add_depth(writer, depth)?;
        writeln!(writer, "}}\n")
    }
}

impl<'a> Function<'a> {
    pub fn with_name(mut self, value: impl Into<Cow<'a, str>>) -> Self {
        self.name = value.into();
        self
    }

    //pub const fn set_const(mut self) -> Self {
    //    self.is_const = true;
    //    self
    //}

    pub const fn with_self_pass(mut self, value: Pass) -> Self {
        self.self_pass = Some(value);
        self
    }

    pub fn with_ret(mut self, value: impl Into<Cow<'a, str>>) -> Self {
        self.ret = Some(value.into());
        self
    }

    pub fn with_body(mut self, value: Body<'a>) -> Self {
        self.body = value;
        self
    }

    pub fn with_arg(mut self, arg: Argument<'a>) -> Self {
        self.args.push(arg);
        self
    }
}

pub struct Argument<'a> {
    pub name: Cow<'a, str>,
    pub kind: Cow<'a, str>,
    pub pass: Pass,
}

impl<'a> Argument<'a> {
    pub fn new(name: impl Into<Cow<'a, str>>, kind: impl Into<Cow<'a, str>>, pass: Pass) -> Self {
        Self {
            name: name.into(),
            kind: kind.into(),
            pass,
        }
    }

    pub fn write_to<W: std::io::Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        let pass = match self.pass {
            Pass::Move | Pass::MutMove => "",
            Pass::Ref => "&",
            Pass::Mut => "&mut ",
        };
        write!(writer, "{}: {}{}", self.name, pass, self.kind)
    }
}

pub struct BodyLine<'a> {
    pub content: Cow<'a, str>,
    pub depth: usize,
}

impl BodyLine<'_> {
    pub fn write_to<W: std::io::Write>(
        &self,
        writer: &mut W,
        depth: usize,
    ) -> Result<(), std::io::Error> {
        add_depth(writer, self.depth + depth)?;
        writeln!(writer, "{}", self.content)
    }
}

#[derive(Default)]
pub struct Body<'a> {
    pub lines: Vec<BodyLine<'a>>,
}

impl<'a> Body<'a> {
    pub fn write_to<W: std::io::Write>(
        &self,
        writer: &mut W,
        depth: usize,
    ) -> Result<(), std::io::Error> {
        for line in &self.lines {
            line.write_to(writer, depth)?;
        }
        Ok(())
    }

    pub fn with_line(&mut self, content: impl Into<Cow<'a, str>>) -> &mut Self {
        self.lines.push(BodyLine {
            content: content.into(),
            depth: 0,
        });
        self
    }

    pub fn with_line_depth(&mut self, content: impl Into<Cow<'a, str>>, depth: usize) -> &mut Self {
        self.lines.push(BodyLine {
            content: content.into(),
            depth,
        });
        self
    }
}
