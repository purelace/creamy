use crate::{
    Diagnostics,
    error::SyntaxError,
    tokenizer::{Span, Token, Tokenizer},
};

enum ControlFlow {
    Continue,
    Skip,
    Break,
    Return,
}

#[derive(Debug)]
pub struct IdentifierNode<'src> {
    pub value: &'src str,
    pub span: Span,
}

#[derive(Debug)]
pub struct NumberNode<'src> {
    pub value: &'src str,
    pub span: Span,
}

#[derive(Debug)]
pub struct StringNode<'src> {
    pub value: &'src str,
    pub span: Span,
}

#[derive(Debug)]
pub enum DefineValueNode<'src> {
    String(StringNode<'src>),
    Number(NumberNode<'src>),
}

#[derive(Debug)]
pub struct DirectiveNode<'src> {
    pub ident: IdentifierNode<'src>,
    pub value: DefineValueNode<'src>,
}

#[derive(Debug, Default)]
pub struct DocumentationNode {
    pub lines: Vec<Span>,
}

#[derive(Debug)]
pub enum AttributeParameterValueNode<'src> {
    Number(NumberNode<'src>),
    String(StringNode<'src>),
}

impl AttributeParameterValueNode<'_> {
    pub const fn span(&self) -> Span {
        match self {
            AttributeParameterValueNode::Number(n) => n.span,
            AttributeParameterValueNode::String(n) => n.span,
        }
    }
}

#[derive(Debug)]
pub struct NamedAttributeParameterNode<'src> {
    pub span: Span,
    pub ident: IdentifierNode<'src>,
    pub value: AttributeParameterValueNode<'src>,
}

#[derive(Debug)]
pub enum AttributeParameterListNode<'src> {
    Unnamed(AttributeParameterValueNode<'src>),
    Named(Vec<NamedAttributeParameterNode<'src>>),
}

#[derive(Debug)]
pub struct AttributeNode<'src> {
    pub span: Span,
    pub ident: IdentifierNode<'src>,
    pub parameters: Option<AttributeParameterListNode<'src>>,
}

#[derive(Debug, Default)]
pub struct AttributeListNode<'src> {
    pub inner: Vec<AttributeNode<'src>>,
}

#[derive(Debug)]
pub struct StaticArrayNode<'src> {
    pub kind: IdentifierNode<'src>,
    pub size: NumberNode<'src>,
}

#[derive(Debug)]
pub struct DynamicArrayNode<'src> {
    pub kind: IdentifierNode<'src>,
}

#[derive(Debug)]
pub enum TypeRefNode<'src> {
    Identifier(IdentifierNode<'src>),
    StaticArray(StaticArrayNode<'src>),
    DynamicArray(Box<TypeRefNode<'src>>),
}

impl<'src> TypeRefNode<'src> {
    pub const fn ident(&self) -> &'src str {
        match self {
            TypeRefNode::Identifier(n) => n.value,
            TypeRefNode::StaticArray(n) => n.kind.value,
            TypeRefNode::DynamicArray(n) => n.ident(),
        }
    }
}

#[derive(Debug)]
pub struct FieldNode<'src> {
    pub ident: IdentifierNode<'src>,
    pub kind: TypeRefNode<'src>,
}

#[derive(Debug)]
pub struct StructNode<'src> {
    pub documentation: DocumentationNode,
    pub attributes: Vec<AttributeListNode<'src>>,
    pub ident: IdentifierNode<'src>,
    pub fields: Vec<FieldNode<'src>>,
}

#[derive(Debug)]
pub struct VariantNode<'src> {
    pub ident: IdentifierNode<'src>,
    pub value: NumberNode<'src>,
}

#[derive(Debug)]
pub struct ModifierNode<'src> {
    pub ident: IdentifierNode<'src>,
}

#[derive(Debug)]
pub struct EnumNode<'src> {
    pub documentation: DocumentationNode,
    pub attributes: Vec<AttributeListNode<'src>>,
    pub modifier: ModifierNode<'src>,
    pub ident: IdentifierNode<'src>,
    pub variants: Vec<VariantNode<'src>>,
}

#[derive(Debug)]
pub struct OptionNode<'src> {
    pub ident: IdentifierNode<'src>,
}

#[derive(Debug)]
pub struct FlagsNode<'src> {
    pub documentation: DocumentationNode,
    pub attributes: Vec<AttributeListNode<'src>>,
    pub ident: IdentifierNode<'src>,
    pub options: Vec<OptionNode<'src>>,
}

#[derive(Debug)]
pub struct BitsetValueTypeNode<'src> {
    pub ident: IdentifierNode<'src>,
    pub width: NumberNode<'src>,
    pub span: Span,
}

#[derive(Debug)]
pub struct BitsetValueNode<'src> {
    pub ident: IdentifierNode<'src>,
    pub kind: BitsetValueTypeNode<'src>,
    pub span: Span,
}

#[derive(Debug)]
pub struct BitsetNode<'src> {
    pub documentation: DocumentationNode,
    pub attributes: Vec<AttributeListNode<'src>>,
    pub ident: IdentifierNode<'src>,
    pub values: Vec<BitsetValueNode<'src>>,
}

#[derive(Debug)]
pub struct ObjectNode<'src> {
    pub documentation: DocumentationNode,
    pub attributes: Vec<AttributeListNode<'src>>,
    pub ident: IdentifierNode<'src>,
    pub fields: Vec<FieldNode<'src>>,
}

#[derive(Debug)]
pub enum MessageNodeType {
    Single,
    Stream,
}

#[derive(Debug)]
pub struct MessageNode<'src> {
    pub documentation: DocumentationNode,
    pub attributes: Vec<AttributeListNode<'src>>,
    pub ident: IdentifierNode<'src>,
    pub modifier: ModifierNode<'src>,
    pub fields: Vec<FieldNode<'src>>,
    pub kind: MessageNodeType,
}

#[derive(Debug)]
pub enum ComplexType<'src> {
    Struct(StructNode<'src>),
    Object(ObjectNode<'src>),
    Message(MessageNode<'src>),
}

impl ComplexType<'_> {
    pub const fn ident(&self) -> &str {
        match self {
            ComplexType::Struct(n) => n.ident.value,
            ComplexType::Object(n) => n.ident.value,
            ComplexType::Message(n) => n.ident.value,
        }
    }

    pub const fn fields(&self) -> &[FieldNode<'_>] {
        match self {
            ComplexType::Struct(n) => n.fields.as_slice(),
            ComplexType::Object(n) => n.fields.as_slice(),
            ComplexType::Message(n) => n.fields.as_slice(),
        }
    }
}

pub struct SyntaxTreeBuilder<'src> {
    source: &'src str,
    tokenizer: Tokenizer<'src>,
    eof_reported: bool,
}

impl<'src> SyntaxTreeBuilder<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            source,
            tokenizer: Tokenizer::new(source),
            eof_reported: false,
        }
    }

    pub fn run(mut self, diag: &mut Diagnostics) -> SyntaxTree<'src> {
        let mut tree = SyntaxTree::default();
        let mut documentation = DocumentationNode::default();
        let mut attributes: Vec<AttributeListNode<'src>> = Vec::new();

        while let Some(token) = self.tokenizer.next() {
            match token {
                // define
                Token::Hash(_) => {
                    tree.directives.push(self.parse_directive_node(diag));
                }

                // keyword
                Token::Ident(span) => match self.slice_for(span) {
                    "flags" => tree.flags.push(self.consume_flags(
                        core::mem::take(&mut documentation),
                        core::mem::take(&mut attributes),
                        diag,
                    )),
                    "enum" => tree.enums.push(self.consume_enum(
                        core::mem::take(&mut documentation),
                        core::mem::take(&mut attributes),
                        diag,
                    )),
                    "bitset" => tree.bitsets.push(self.consume_bitset(
                        core::mem::take(&mut documentation),
                        core::mem::take(&mut attributes),
                        diag,
                    )),
                    "struct" => tree.complex.push(self.consume_struct(
                        core::mem::take(&mut documentation),
                        core::mem::take(&mut attributes),
                        diag,
                    )),
                    "object" => tree.complex.push(self.consume_object(
                        core::mem::take(&mut documentation),
                        core::mem::take(&mut attributes),
                        diag,
                    )),
                    "message" => tree.complex.push(self.consume_message(
                        core::mem::take(&mut documentation),
                        core::mem::take(&mut attributes),
                        diag,
                    )),
                    "stream" => tree.complex.push(self.consume_stream(
                        core::mem::take(&mut documentation),
                        core::mem::take(&mut attributes),
                        diag,
                    )),
                    other => {
                        diag.report_err(SyntaxError::UnknownKeyword {
                            actual: other.to_string(),
                            span: span.into(),
                        });
                    }
                },

                // docs
                Token::Documentation(span) => documentation.lines.push(span),
                // attribute
                Token::LBracket(_) => {
                    attributes.push(self.parse_attribute_node(diag));
                }

                Token::Unknown(_) => todo!(),
                u => {
                    //diag.report_err(SyntaxError::UnexpectedToken {
                    //    expected: "keyword, documentation or attribute",
                    //    span: u.span().into(),
                    //});
                    todo!("unexpected token: {u:#?}")
                }
            }
        }

        tree
    }

    fn slice_for(&self, span: Span) -> &'src str {
        &self.source[span.as_range()]
    }

    fn consume_ident(&mut self, diag: &mut Diagnostics) -> IdentifierNode<'src> {
        let token = self.tokenizer.next();
        match token {
            Some(Token::Ident(span)) => IdentifierNode {
                value: self.slice_for(span),
                span,
            },
            Some(other) => {
                todo!("report an unexpected token error and return broken node ({other:#?})");
            }
            None => todo!("end of file"),
        }
    }

    fn consume_define_value_node(&mut self, diag: &mut Diagnostics) -> DefineValueNode<'src> {
        match self.tokenizer.next() {
            Some(Token::String(span)) => DefineValueNode::String(StringNode {
                value: self.slice_for(span),
                span,
            }),
            Some(Token::Number(span)) => DefineValueNode::Number(NumberNode {
                value: self.slice_for(span),
                span,
            }),
            Some(other) => {
                todo!("report an unexpected token error and return broken node");
            }
            None => todo!("end of file"),
        }
    }

    fn parse_directive_node(&mut self, diag: &mut Diagnostics) -> DirectiveNode<'src> {
        DirectiveNode {
            ident: self.consume_ident(diag),
            value: self.consume_define_value_node(diag),
        }
    }

    fn consume_rparen(&mut self, diag: &mut Diagnostics) {
        match self.tokenizer.next() {
            Some(Token::RParen(_)) => {}
            Some(other) => todo!(),
            None => todo!("end of file"),
        }
    }

    fn consume_equals(&mut self, diag: &mut Diagnostics) -> bool {
        match self.tokenizer.next() {
            Some(Token::Equals(_)) => true,
            Some(other) => {
                diag.report_err(SyntaxError::UnexpectedToken {
                    expected: "=",
                    span: other.span().into(),
                });
                false
            }
            None => {
                diag.report_err(SyntaxError::UnexpectedEOF);
                false
            }
        }
    }

    fn consume_named_parameter_node(
        &mut self,
        ident_span: Span,
        diag: &mut Diagnostics,
    ) -> NamedAttributeParameterNode<'src> {
        let ident = IdentifierNode {
            value: self.slice_for(ident_span),
            span: ident_span,
        };

        self.consume_equals(diag);

        match self.tokenizer.next() {
            Some(Token::Number(span)) => NamedAttributeParameterNode {
                span: Span::extend(ident.span, span),
                ident,
                value: AttributeParameterValueNode::Number(NumberNode {
                    value: self.slice_for(span),
                    span,
                }),
            },
            Some(Token::String(span)) => NamedAttributeParameterNode {
                span: Span::extend(ident.span, span),
                ident,
                value: AttributeParameterValueNode::String(StringNode {
                    value: self.slice_for(span),
                    span,
                }),
            },
            Some(other) => todo!(),
            None => todo!("end of file"),
        }
    }

    fn consume_attribute_parameter_list_node(
        &mut self,
        diag: &mut Diagnostics,
    ) -> AttributeParameterListNode<'src> {
        match self.tokenizer.next() {
            Some(Token::RParen(_)) => AttributeParameterListNode::Named(Vec::new()),
            Some(Token::Number(span)) => {
                self.consume_rparen(diag);
                AttributeParameterListNode::Unnamed(AttributeParameterValueNode::Number(
                    NumberNode {
                        value: self.slice_for(span),
                        span,
                    },
                ))
            }
            Some(Token::String(span)) => {
                self.consume_rparen(diag);
                AttributeParameterListNode::Unnamed(AttributeParameterValueNode::String(
                    StringNode {
                        value: self.slice_for(span),
                        span,
                    },
                ))
            }
            Some(Token::Ident(span)) => {
                let mut inner = vec![self.consume_named_parameter_node(span, diag)];
                loop {
                    match self.tokenizer.next() {
                        Some(Token::Ident(span)) => {
                            inner.push(self.consume_named_parameter_node(span, diag));
                            match self.tokenizer.next() {
                                Some(Token::RParen(_)) => {
                                    return AttributeParameterListNode::Named(inner);
                                }
                                Some(Token::Comma(_)) => {}
                                Some(other) => todo!("other: {other:#?}"),
                                None => todo!("end of file"),
                            }
                        }
                        Some(Token::RParen(_)) => return AttributeParameterListNode::Named(inner),
                        Some(Token::Comma(_)) => {
                            /* TODO: этого тут не должно быть */
                        }
                        Some(other) => todo!("other: {other:#?}"),
                        None => todo!("end of file"),
                    }
                }
            }
            Some(other) => todo!(),
            None => todo!("end of file"),
        }
    }

    fn parse_attribute_node(&mut self, diag: &mut Diagnostics) -> AttributeListNode<'src> {
        let mut list = AttributeListNode::default();

        loop {
            let ident = match self.tokenizer.next() {
                Some(Token::RBracket(_)) => break,
                Some(Token::Ident(span)) => IdentifierNode {
                    value: self.slice_for(span),
                    span,
                },
                Some(Token::Comma(_)) => continue,
                Some(other) => todo!("other: {other:#?}"),
                None => todo!("end of file"),
            };

            match self.tokenizer.next() {
                Some(Token::Comma(_)) => {
                    list.inner.push(AttributeNode {
                        span: ident.span,
                        ident,
                        parameters: None,
                    });
                }
                Some(Token::RBracket(_)) => {
                    list.inner.push(AttributeNode {
                        span: ident.span,
                        ident,
                        parameters: None,
                    });
                    break;
                }
                Some(Token::LParen(_)) => {
                    let parameters = self.consume_attribute_parameter_list_node(diag);
                    list.inner.push(AttributeNode {
                        span: ident.span,
                        ident,
                        parameters: Some(parameters),
                    })
                }
                Some(other) => todo!("other: {other:#?}"),
                None => todo!("end of file"),
            }
        }

        list
    }

    fn consume_lbrace(&mut self, diag: &mut Diagnostics) {
        match self.tokenizer.next() {
            Some(Token::LBrace(_)) => {}
            Some(other) => todo!(),
            None => todo!("end of file"),
        }
    }

    fn consume_rbrace(&mut self, diag: &mut Diagnostics) {
        match self.tokenizer.next() {
            Some(Token::RBrace(_)) => {}
            Some(other) => todo!(),
            None => todo!("end of file"),
        }
    }

    fn consume_comma(&mut self, diag: &mut Diagnostics) {
        match self.tokenizer.next() {
            Some(Token::Comma(_)) => {}
            Some(other) => todo!(),
            None => todo!("end of file"),
        }
    }

    fn consume_option_list_node(&mut self, diag: &mut Diagnostics) -> Vec<OptionNode<'src>> {
        let mut inner = Vec::new();

        loop {
            match self.tokenizer.next() {
                Some(Token::Ident(span)) => {
                    inner.push(OptionNode {
                        ident: IdentifierNode {
                            value: self.slice_for(span),
                            span,
                        },
                    });

                    match self.tokenizer.next() {
                        Some(Token::RBrace(_)) => break,
                        Some(Token::Comma(_)) => {}
                        Some(other) => todo!("other: {other:#?}"),
                        None => todo!("end of file"),
                    }
                }
                Some(Token::RBrace(_)) => break,
                Some(other) => todo!("other: {other:#?}"),
                None => todo!("end of file"),
            }
        }

        inner
    }

    fn consume_flags(
        &mut self,
        documentation: DocumentationNode,
        attributes: Vec<AttributeListNode<'src>>,
        diag: &mut Diagnostics,
    ) -> FlagsNode<'src> {
        let ident = self.consume_ident(diag);
        self.consume_lbrace(diag);
        let options = self.consume_option_list_node(diag);

        FlagsNode {
            documentation,
            attributes,
            ident,
            options,
        }
    }

    fn consume_lbracket(&mut self, diag: &mut Diagnostics) {
        match self.tokenizer.next() {
            Some(Token::LBracket(_)) => {}
            Some(other) => todo!(),
            None => todo!("end of file"),
        }
    }

    fn consume_rbracket(&mut self, diag: &mut Diagnostics) {
        match self.tokenizer.next() {
            Some(Token::RBracket(_)) => {}
            Some(other) => todo!(),
            None => todo!("end of file"),
        }
    }

    fn consume_modifier(&mut self, diag: &mut Diagnostics) -> ModifierNode<'src> {
        self.consume_lbracket(diag);
        let node = ModifierNode {
            ident: self.consume_ident(diag),
        };
        self.consume_rbracket(diag);
        node
    }

    fn consume_number(&mut self, diag: &mut Diagnostics) -> NumberNode<'src> {
        match self.tokenizer.next() {
            Some(Token::Number(span)) => NumberNode {
                value: self.slice_for(span),
                span,
            },
            Some(other) => todo!("other: {other:#?}"),
            None => todo!("end of file"),
        }
    }

    fn skip_until(&mut self, slice: &[Token]) -> Option<Token> {
        self.tokenizer
            .by_ref()
            .find(|token| slice.iter().any(|t| t.matches(token)))
    }

    fn report_eof(&mut self, diag: &mut Diagnostics) {
        if self.eof_reported {
            return;
        }

        self.eof_reported = true;
        diag.report_err(SyntaxError::UnexpectedEOF);
    }

    fn consume_equals_or_skip(&mut self, diag: &mut Diagnostics) -> ControlFlow {
        if self.consume_equals(diag) {
            return ControlFlow::Continue;
        }

        match self.skip_until(&[
            Token::RBrace(Span::default()),
            Token::Comma(Span::default()),
        ]) {
            Some(Token::RBrace(_)) => ControlFlow::Break,
            Some(Token::Comma(_)) => ControlFlow::Skip,
            Some(_) => unreachable!(),
            None => {
                self.report_eof(diag);
                ControlFlow::Return
            }
        }
    }

    fn consume_variants_list_node(&mut self, diag: &mut Diagnostics) -> Vec<VariantNode<'src>> {
        let mut inner = Vec::new();

        loop {
            match self.tokenizer.next() {
                Some(Token::Ident(span)) => {
                    let ident = IdentifierNode {
                        value: self.slice_for(span),
                        span,
                    };

                    match self.consume_equals_or_skip(diag) {
                        ControlFlow::Continue => {}
                        ControlFlow::Skip => continue,
                        ControlFlow::Break => break,
                        ControlFlow::Return => return inner,
                    }

                    let value = self.consume_number(diag);
                    inner.push(VariantNode { ident, value });

                    match self.tokenizer.next() {
                        Some(Token::RBrace(_)) => break,
                        Some(Token::Comma(_)) => {}
                        Some(other) => todo!("other: {other:#?}"),
                        None => todo!("end of file"),
                    }
                }
                Some(Token::RBrace(_)) => break,
                Some(other) => todo!("other: {other:#?}"),
                None => todo!("end of file"),
            }
        }

        inner
    }

    fn consume_enum(
        &mut self,
        documentation: DocumentationNode,
        attributes: Vec<AttributeListNode<'src>>,
        diag: &mut Diagnostics,
    ) -> EnumNode<'src> {
        let modifier = self.consume_modifier(diag);
        let ident = self.consume_ident(diag);
        self.consume_lbrace(diag);
        let variants = self.consume_variants_list_node(diag);

        EnumNode {
            documentation,
            attributes,
            modifier,
            ident,
            variants,
        }
    }

    fn consume_colon(&mut self, diag: &mut Diagnostics) {
        match self.tokenizer.next() {
            Some(Token::Colon(_)) => {}
            Some(other) => todo!(),
            None => todo!("end of file"),
        }
    }

    fn consume_bitset_value_list_node(
        &mut self,
        diag: &mut Diagnostics,
    ) -> Vec<BitsetValueNode<'src>> {
        let mut inner = Vec::new();

        loop {
            match self.tokenizer.next() {
                Some(Token::Ident(span)) => {
                    let ident = IdentifierNode {
                        value: self.slice_for(span),
                        span,
                    };

                    self.consume_colon(diag);
                    let kind = self.consume_ident(diag);
                    self.consume_colon(diag);
                    let width = self.consume_number(diag);

                    let kind = BitsetValueTypeNode {
                        span: Span::extend(kind.span, width.span),
                        ident: kind,
                        width,
                    };

                    inner.push(BitsetValueNode {
                        span: Span::extend(ident.span, kind.span),
                        ident,
                        kind,
                    });

                    match self.tokenizer.next() {
                        Some(Token::RBrace(_)) => break,
                        Some(Token::Comma(_)) => {}
                        Some(other) => todo!(),
                        None => todo!("end of file"),
                    }
                }
                Some(Token::RBrace(_)) => break,
                Some(other) => todo!("other: {other:#?}"),
                None => todo!("end of file"),
            }
        }

        inner
    }

    fn consume_bitset(
        &mut self,
        documentation: DocumentationNode,
        attributes: Vec<AttributeListNode<'src>>,
        diag: &mut Diagnostics,
    ) -> BitsetNode<'src> {
        let ident = self.consume_ident(diag);
        self.consume_lbrace(diag);
        let values = self.consume_bitset_value_list_node(diag);

        BitsetNode {
            documentation,
            attributes,
            ident,
            values,
        }
    }

    fn consume_langle(&mut self, diag: &mut Diagnostics) -> bool {
        match self.tokenizer.next() {
            Some(Token::LAngle(_)) => true,
            Some(other) => {
                todo!("other: {other:#?}");
            }
            None => {
                diag.report_err(SyntaxError::UnexpectedEOF);
                false
            }
        }
    }

    fn consume_rangle(&mut self, diag: &mut Diagnostics) {
        match self.tokenizer.next() {
            Some(Token::RAngle(_)) => {}
            Some(other) => todo!(),
            None => todo!("end of file"),
        }
    }

    fn consume_semicolon(&mut self, diag: &mut Diagnostics) {
        match self.tokenizer.next() {
            Some(Token::Semicolon(_)) => {}
            Some(other) => todo!(),
            None => todo!("end of file"),
        }
    }

    fn consume_type_ref_node(&mut self, diag: &mut Diagnostics) -> TypeRefNode<'src> {
        match self.tokenizer.next() {
            Some(Token::Ident(span)) => {
                let ident = self.slice_for(span);
                if ident == "Array" {
                    self.consume_langle(diag);
                    let kind = self.consume_type_ref_node(diag);
                    self.consume_rangle(diag);
                    TypeRefNode::DynamicArray(Box::new(kind))
                } else {
                    TypeRefNode::Identifier(IdentifierNode { value: ident, span })
                }
            }
            Some(Token::LBracket(_)) => {
                let kind = self.consume_ident(diag);
                self.consume_semicolon(diag);
                let size = self.consume_number(diag);
                self.consume_rbracket(diag);
                TypeRefNode::StaticArray(StaticArrayNode { kind, size })
            }
            Some(other) => todo!("other: {other:#?}"),
            None => todo!("end of file"),
        }
    }

    fn consume_field_list(&mut self, diag: &mut Diagnostics) -> Vec<FieldNode<'src>> {
        let mut inner = Vec::new();

        loop {
            match self.tokenizer.next() {
                Some(Token::Ident(span)) => {
                    let ident = IdentifierNode {
                        value: self.slice_for(span),
                        span,
                    };
                    self.consume_colon(diag);
                    let kind = self.consume_type_ref_node(diag);

                    inner.push(FieldNode { ident, kind });

                    match self.tokenizer.next() {
                        Some(Token::RBrace(_)) => break,
                        Some(Token::Comma(_)) => {}
                        Some(other) => todo!("other: {other:#?}"),
                        None => todo!("end of file"),
                    }
                }
                Some(Token::RBrace(_)) => break,
                Some(other) => todo!("other: {other:#?}"),
                None => todo!("end of file"),
            }
        }

        inner
    }

    fn consume_struct(
        &mut self,
        documentation: DocumentationNode,
        attributes: Vec<AttributeListNode<'src>>,
        diag: &mut Diagnostics,
    ) -> ComplexType<'src> {
        let ident = self.consume_ident(diag);
        self.consume_lbrace(diag);
        let fields = self.consume_field_list(diag);

        ComplexType::Struct(StructNode {
            documentation,
            attributes,
            ident,
            fields,
        })
    }

    fn consume_object(
        &mut self,
        documentation: DocumentationNode,
        attributes: Vec<AttributeListNode<'src>>,
        diag: &mut Diagnostics,
    ) -> ComplexType<'src> {
        let ident = self.consume_ident(diag);
        self.consume_lbrace(diag);
        let fields = self.consume_field_list(diag);

        ComplexType::Object(ObjectNode {
            documentation,
            attributes,
            ident,
            fields,
        })
    }

    fn consume_message(
        &mut self,
        documentation: DocumentationNode,
        attributes: Vec<AttributeListNode<'src>>,
        diag: &mut Diagnostics,
    ) -> ComplexType<'src> {
        let modifier = self.consume_modifier(diag);
        let ident = self.consume_ident(diag);
        self.consume_lbrace(diag);
        let fields = self.consume_field_list(diag);

        ComplexType::Message(MessageNode {
            documentation,
            attributes,
            ident,
            modifier,
            fields,
            kind: MessageNodeType::Single,
        })
    }

    fn consume_stream(
        &mut self,
        documentation: DocumentationNode,
        attributes: Vec<AttributeListNode<'src>>,
        diag: &mut Diagnostics,
    ) -> ComplexType<'src> {
        let modifier = self.consume_modifier(diag);
        let ident = self.consume_ident(diag);
        self.consume_lbrace(diag);
        let fields = self.consume_field_list(diag);

        ComplexType::Message(MessageNode {
            documentation,
            attributes,
            ident,
            modifier,
            fields,
            kind: MessageNodeType::Stream,
        })
    }
}

#[derive(Default, Debug)]
pub struct SyntaxTree<'src> {
    pub directives: Vec<DirectiveNode<'src>>,
    pub flags: Vec<FlagsNode<'src>>,
    pub enums: Vec<EnumNode<'src>>,
    pub bitsets: Vec<BitsetNode<'src>>,
    pub complex: Vec<ComplexType<'src>>,
}

impl SyntaxTree<'_> {
    pub fn type_count(&self) -> usize {
        self.flags.len()
            + self.enums.len()
            + self.bitsets.len()
            + self
                .complex
                .iter()
                .map(|n| usize::from(matches!(n, ComplexType::Struct(_) | ComplexType::Object(_))))
                .sum::<usize>()
    }
}
