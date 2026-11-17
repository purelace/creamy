use semver::Version;

use super::{Context, parse_version, utils::consume_string};
use crate::{
    ast::{AttributeListNode, AttributeParameterListNode, NamedAttributeParameterNode},
    error::SemanticError,
    symbols::ApiStatus,
};

#[derive(Default)]
pub enum Value<T> {
    #[default]
    None,
    Error,
    Some(T),
}

impl<T> Value<T> {
    const fn is_some(&self) -> bool {
        match self {
            Value::None => false,
            Value::Error | Value::Some(_) => true,
        }
    }
}

fn consume_version(context: &mut Context, node: &NamedAttributeParameterNode) -> Value<Version> {
    if let Some((v, s)) = consume_string(context, &node.value) {
        Value::Some(parse_version(&mut context.diag, v, s))
    } else {
        Value::Error
    }
}

#[derive(Default)]
pub struct Metadata {
    from: Value<Version>,
    until: Value<Version>,
    reason: Option<String>,
}

fn consume_metadata(context: &mut Context, node: Option<&AttributeParameterListNode>) -> Metadata {
    let mut metadata = Metadata::default();
    if let Some(params) = &node {
        match params {
            AttributeParameterListNode::Unnamed(_) => {
                todo!("unsupported")
                //return None;
            }
            AttributeParameterListNode::Named(n) => {
                for n in n {
                    match n.ident.value {
                        "from" => {
                            if metadata.from.is_some() {
                                todo!("parameter already defined");
                                //continue;
                            }

                            // TODO: version can't be lower than protocol version
                            // TODO: version can't be more than until version
                            metadata.from = consume_version(context, n);
                        }
                        "until" => {
                            if metadata.until.is_some() {
                                todo!("parameter already defined");
                                //continue;
                            }

                            metadata.until = consume_version(context, n);
                        }
                        "reason" => {
                            if metadata.reason.is_some() {
                                todo!("parameter already defined");
                            }

                            metadata.reason =
                                if let Some((v, _)) = consume_string(context, &n.value) {
                                    Some(v.to_string())
                                } else {
                                    Some(String::new())
                                };
                        }
                        _ => todo!("unknown parameter"),
                    }
                }
                metadata
            }
        }
    } else {
        metadata
    }
}

#[derive(Default)]
pub struct Attributes {
    pub extension: Option<Metadata>,
    pub experimental: Option<Metadata>,
    pub builtin: bool,
}

impl Attributes {
    pub fn get_api_status(&self, context: &mut Context) -> ApiStatus {
        match (&self.experimental, &self.extension) {
            (Some(_), Some(_)) => context
                .diag
                .report_err(SemanticError::ConflictingAttributes {
                    a: "Experimental",
                    b: "Extension",
                }),
            (Some(_), None) => ApiStatus::Experimental,
            (None, Some(_)) => ApiStatus::Extension,
            (None, None) => ApiStatus::Stable,
        }
    }
}

pub fn resolve_attribute_list(context: &mut Context, list: &[AttributeListNode]) -> Attributes {
    let mut attributes = Attributes::default();

    for root in list.iter().flat_map(|n| &n.inner) {
        match root.ident.value {
            "Extension" => {
                if attributes.extension.is_some() {
                    context
                        .diag
                        .report_err::<()>(SemanticError::DuplicateAttributeError {
                            name: "Extension",
                            span: root.ident.span.into(),
                        });
                    continue;
                }

                attributes.extension = Some(consume_metadata(context, root.parameters.as_ref()));
            }
            "Experimental" => {
                if attributes.experimental.is_some() {
                    context
                        .diag
                        .report_err::<()>(SemanticError::DuplicateAttributeError {
                            name: "Experimental",
                            span: root.ident.span.into(),
                        });
                    continue;
                }

                attributes.experimental = Some(consume_metadata(context, root.parameters.as_ref()));
            }
            "Builtin" => {
                if attributes.builtin {
                    context
                        .diag
                        .report_err::<()>(SemanticError::DuplicateAttributeError {
                            name: "Builtin",
                            span: root.ident.span.into(),
                        });
                    continue;
                }

                attributes.builtin = true;
                if root.parameters.is_some() {
                    panic!("'Builtin' attribute doesn't accept any parameter")
                }
            }
            other => {
                context
                    .diag
                    .report_err::<()>(SemanticError::UnknownAttribute {
                        name: other.to_string(),
                        span: root.ident.span.into(),
                    });
            }
        }
    }

    attributes
}
