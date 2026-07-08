use std::borrow::Cow;

use crate::generator::{
    Argument, Body, BodyLine, Function, Pass, TraitImpl, TraitImplAssociatedType,
};

pub fn generate_bitxor_impl<'a>(target: &'a str, kind: &'a str) -> TraitImpl<'a> {
    TraitImpl {
        trait_name: Cow::Borrowed("core::ops::BitXor"),
        associated_types: vec![TraitImplAssociatedType {
            name: "Output",
            kind: kind.into(),
        }],
        target: target.into(),
        functions: vec![
            Function::default()
                .with_name("bitxor")
                .with_self_pass(Pass::Move)
                .with_arg(Argument {
                    name: "rhs",
                    kind: Cow::Borrowed("Self"),
                    pass: Pass::Move,
                })
                .with_ret("Self::Output")
                .with_body(Body {
                    lines: vec![BodyLine {
                        content: Cow::Owned(format!("{kind}::bitxor(self.0, rhs.0)")),
                        depth: 0,
                    }],
                }),
        ],
        constants: vec![],
    }
}

pub fn generate_bitor_impl<'a>(target: &'a str, kind: &'a str) -> TraitImpl<'a> {
    TraitImpl {
        trait_name: Cow::Borrowed("core::ops::BitOr"),
        associated_types: vec![TraitImplAssociatedType {
            name: "Output",
            kind: kind.into(),
        }],
        target: target.into(),
        functions: vec![
            Function::default()
                .with_name("bitor")
                .with_self_pass(Pass::Move)
                .with_arg(Argument {
                    name: "rhs",
                    kind: Cow::Borrowed("Self"),
                    pass: Pass::Move,
                })
                .with_ret("Self::Output")
                .with_body(Body {
                    lines: vec![BodyLine {
                        content: Cow::Owned(format!("{kind}::bitor(self.0, rhs.0)")),
                        depth: 0,
                    }],
                }),
        ],
        constants: vec![],
    }
}
