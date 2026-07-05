use crate::generator::{CodeBlock, Function, add_depth};

pub struct Impl<'a, W: std::io::Write> {
    pub target: &'a str,
    pub functions: Vec<Function<'a>>,
    pub add_assert: bool,
    pub custom: Vec<Box<dyn CodeBlock<W> + 'a>>,
}

impl<W: std::io::Write> CodeBlock<W> for Impl<'_, W> {
    fn write_to(&self, writer: &mut W, depth: usize) -> Result<(), std::io::Error> {
        add_depth(writer, depth)?;
        writeln!(writer, "impl {} {{", self.target)?;

        for block in &self.custom {
            block.write_to(writer, depth + 1)?;
        }

        //if self.add_assert {
        //    add_depth(writer, depth + 1)?;
        //    writeln!(writer, "const __ASSERT_CHECK_SIZE: () = {{")?;
        //    add_depth(writer, depth + 2)?;
        //    writeln!(
        //        writer,
        //        "assert!(size_of::<{}>() == {MESSAGE_SIZE});",
        //        self.target
        //    )?;
        //    add_depth(writer, depth + 1)?;
        //    writeln!(writer, "}};\n")?;
        //}

        for function in &self.functions {
            function.write_to(writer, depth + 1)?;
        }

        add_depth(writer, depth)?;
        writeln!(writer, "}}\n")
    }
}
