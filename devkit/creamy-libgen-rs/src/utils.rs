use std::{borrow::Cow, io::Write, marker::PhantomData};

use crate::{
    Args,
    generator::{Access, CodeBlock, Const, add_depth},
};

struct ConstAssertBlock<W: Write> {
    _phantom: PhantomData<W>,
    //FIX: use Cow instead
    size: String,
    target: String,
}

impl<W: Write> CodeBlock<W> for ConstAssertBlock<W> {
    fn write_to(&self, writer: &mut W, depth: usize) -> Result<(), std::io::Error> {
        add_depth(writer, depth)?;
        writeln!(writer, "const _: () = {{")?;
        add_depth(writer, depth + 1)?;
        writeln!(
            writer,
            "assert!(size_of::<{current_type}>() == {size});",
            current_type = self.target,
            size = self.size,
        )?;
        add_depth(writer, depth)?;
        writeln!(writer, "}};\n")?;
        Ok(())
    }
}

pub fn generate_const_size_assert<'s, W: Write + 's>(
    size: String,
    target: String,
) -> Box<dyn CodeBlock<W> + 's> {
    Box::new(ConstAssertBlock {
        _phantom: PhantomData,
        size,
        target,
    })
}

pub fn generate_message_consts<'s, W: Write + 's>(
    args: &Args,
    group: u8,
    kind: u8,
    dispatch_value: u32,
) -> Vec<Box<dyn CodeBlock<W> + 's>> {
    vec![
        Box::new(generate_message_group_const(group)),
        Box::new(generate_message_kind_const(kind)),
        Box::new(generate_zeroed_message_const(args)),
        Box::new(generate_prepared_message_const()),
        Box::new(generate_dispatch_value_const(dispatch_value)),
    ]
}

pub fn generate_message_group_const(group: u8) -> Const<'static> {
    Const {
        access: Access::Pub,
        ident: Cow::Borrowed("GROUP"),
        kind: Cow::Borrowed("u8"),
        value: Cow::Owned(group.to_string()),
    }
}

pub fn generate_message_kind_const(kind: u8) -> Const<'static> {
    Const {
        access: Access::Pub,
        ident: Cow::Borrowed("KIND"),
        kind: Cow::Borrowed("u8"),
        value: Cow::Owned(kind.to_string()),
    }
}

pub fn generate_zeroed_message_const<'s>(args: &Args) -> Const<'s> {
    Const {
        access: Access::Pub,
        ident: "ZEROED".into(),
        kind: "Self".into(),
        //TODO: better format
        value: format!(
            "
            unsafe {{
                core::mem::transmute::<_, Self>([0u8; {message_size}])
            }}
        ",
            message_size = args.message_size_path()
        )
        .into(),
    }
}

pub fn generate_prepared_message_const<'s>() -> Const<'s> {
    Const {
        access: Access::Pub,
        ident: "PREPARED".into(),
        kind: "Self".into(),
        value: "{
            let mut value = Self::ZEROED;
            value.group = Self::GROUP;
            value.kind = Self::KIND;
            value
        }"
        .into(),
    }
}

pub fn generate_dispatch_value_const<'s>(dispatch_value: u32) -> Const<'s> {
    Const {
        access: Access::Pub,
        ident: "DISPATCH_VALUE".into(),
        kind: "u32".into(),
        value: dispatch_value.to_string().into(),
    }
}
