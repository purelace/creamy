use std::borrow::Cow;

use creamy_libgen::{
    SymbolIterator,
    proxy::{EnrichedFieldSymbol, EnrichedFieldType},
};

use crate::generator::{Access, Argument, Body, BodyLine, Function, Impl, Pass};

pub fn generate_builder_pattern<'s, W, I>(fields: I, message: &'s str) -> Impl<'s, W>
where
    W: std::io::Write,
    I: SymbolIterator<EnrichedFieldSymbol<'s>>,
{
    let functions: Vec<Function<'_>> = fields
        .filter_map(|f| {
            if f.is_padding {
                return None;
            }

            let field_name = &f.name;
            let kind = match f.kind {
                EnrichedFieldType::Type(cow) => cow,
                EnrichedFieldType::Array { kind, len } => format!("[{kind}; {len}]").into(),
            };

            Some((
                Function {
                    access: Access::Pub,
                    is_const: true,
                    name: Cow::Owned(format!("set_{field_name}")),
                    self_pass: Some(Pass::Mut),
                    arg: vec![Argument {
                        name: "value",
                        kind: kind.clone(),
                        pass: Pass::Move,
                    }],
                    ret: Some(Cow::Borrowed("&mut Self")),
                    body: Body {
                        lines: vec![
                            BodyLine {
                                content: Cow::Owned(format!("self.{field_name} = value;")),
                                depth: 0,
                            },
                            BodyLine {
                                content: Cow::Borrowed("self"),
                                depth: 0,
                            },
                        ],
                    },
                },
                Function {
                    access: Access::Pub,
                    is_const: true,
                    name: Cow::Owned(format!("with_{field_name}")),
                    self_pass: Some(Pass::MutMove),
                    arg: vec![Argument {
                        name: "value",
                        kind,
                        pass: Pass::Move,
                    }],
                    ret: Some(Cow::Borrowed("Self")),
                    body: Body {
                        lines: vec![
                            BodyLine {
                                content: Cow::Owned(format!("self.set_{field_name}(value);")),
                                depth: 0,
                            },
                            BodyLine {
                                content: Cow::Borrowed("self"),
                                depth: 0,
                            },
                        ],
                    },
                },
            ))
        })
        .flat_map(|(a, b)| [a, b])
        .collect();

    Impl {
        target: message.into(),
        functions,
        custom: vec![],
    }
}
