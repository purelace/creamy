use std::borrow::Cow;

use super::{Access, Pass, add_depth};

#[derive(Default)]
pub struct Function<'a> {
    pub access: Access,
    pub is_const: bool,
    pub name: Cow<'a, str>,
    pub self_pass: Option<Pass>,
    pub arg: Vec<Argument<'a>>,
    pub ret: Option<Cow<'a, str>>,
    pub body: Body<'a>,
}

impl<'a> Function<'a> {
    pub fn write_to<W: std::io::Write>(
        &self,
        writer: &mut W,
        depth: usize,
    ) -> Result<(), std::io::Error> {
        add_depth(writer, depth)?;
        write!(writer, "{}", self.access)?;
        if self.is_const {
            write!(writer, " const ")?;
        }
        write!(writer, "fn {}(", self.name)?;

        if let Some(pass) = self.self_pass {
            let arg = match pass {
                Pass::Move => "self",
                Pass::Ref => "&self",
                Pass::Mut => "&mut self",
            };
            write!(writer, "{arg}")?;
        }

        if !self.arg.is_empty() {
            write!(writer, ", ")?;
            for (idx, arg) in self.arg.iter().enumerate() {
                arg.write_to(writer)?;
                if idx != self.arg.len() - 1 {
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
        self.arg.push(arg);
        self
    }
}

pub struct Argument<'a> {
    pub name: &'a str,
    pub kind: Cow<'a, str>,
    pub pass: Pass,
}

impl Argument<'_> {
    pub fn write_to<W: std::io::Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        write!(writer, "{}: {}{}", self.name, self.pass, self.kind)
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
