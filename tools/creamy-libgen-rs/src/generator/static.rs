use std::borrow::Cow;

use super::{Access, CodeBlock};
use crate::generator::add_depth;

pub struct Static<'a> {
    pub access: Access,
    pub name: Cow<'a, str>,
    pub kind: Cow<'a, str>,
    pub value: Cow<'a, str>,
}

impl<W: std::io::Write> CodeBlock<W> for Static<'_> {
    fn write_to(&self, writer: &mut W, depth: usize) -> Result<(), std::io::Error> {
        add_depth(writer, depth)?;
        writeln!(
            writer,
            "{} static {}: {} = {};\n",
            self.access, self.name, self.kind, self.value
        )
    }
}
