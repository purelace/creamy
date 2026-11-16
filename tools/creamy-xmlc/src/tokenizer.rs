use std::str::Chars;

use miette::SourceSpan;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    offset: usize,
    length: usize,
}

impl From<Span> for SourceSpan {
    fn from(val: Span) -> Self {
        SourceSpan::new(val.offset.into(), val.length)
    }
}

impl Span {
    pub const fn as_range(&self) -> core::ops::Range<usize> {
        self.offset..self.offset + self.length
    }

    pub fn extend(a: Self, b: Self) -> Self {
        let max_point_a = a.offset + a.length;
        let max_point_b = b.offset + b.length;
        let max_point = max_point_a.max(max_point_b);
        let offset = a.offset.min(b.offset);
        let length = max_point - offset;
        Span { offset, length }
    }
}

pub enum Token {
    Ident(Span),
    /// "..."
    String(Span),
    /// -0123456789
    Number(Span),
    /// ///
    Documentation(Span),
    /// #
    Hash(Span),
    /// {
    LBrace(Span),
    /// }
    RBrace(Span),
    /// [
    LBracket(Span),
    /// ]
    RBracket(Span),
    /// (
    LParen(Span),
    /// )
    RParen(Span),
    /// <
    LAngle(Span),
    /// \>
    RAngle(Span),
    /// :
    Colon(Span),
    /// ;
    Semicolon(Span),
    /// ,
    Comma(Span),
    /// =
    Equals(Span),
    Unknown((char, Span)),
}

impl core::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ident(_) => f.debug_tuple("Ident").finish(),
            Self::String(_) => f.debug_tuple("String").finish(),
            Self::Number(_) => f.debug_tuple("Number").finish(),
            Self::Documentation(_) => f.debug_tuple("Documentation").finish(),
            Self::Hash(_) => f.debug_tuple("Hash").finish(),
            Self::LBrace(_) => f.debug_tuple("LBrace").finish(),
            Self::RBrace(_) => f.debug_tuple("RBrace").finish(),
            Self::LBracket(_) => f.debug_tuple("LBracket").finish(),
            Self::RBracket(_) => f.debug_tuple("RBracket").finish(),
            Self::LParen(_) => f.debug_tuple("LParen").finish(),
            Self::RParen(_) => f.debug_tuple("RParen").finish(),
            Self::LAngle(_) => f.debug_tuple("LAngle").finish(),
            Self::RAngle(_) => f.debug_tuple("RAngle").finish(),
            Self::Colon(_) => f.debug_tuple("Colon").finish(),
            Self::Semicolon(_) => f.debug_tuple("Semicolon").finish(),
            Self::Comma(_) => f.debug_tuple("Comma").finish(),
            Self::Equals(_) => f.debug_tuple("Equals").finish(),
            Self::Unknown(_) => f.debug_tuple("Unknown").finish(),
        }
    }
}

impl Token {
    pub const fn matches(&self, rhs: &Self) -> bool {
        matches!(
            (self, rhs),
            (Self::Ident(_), Self::Ident(_))
                | (Self::String(_), Self::String(_))
                | (Self::Number(_), Self::Number(_))
                | (Self::Documentation(_), Self::Documentation(_))
                | (Self::Hash(_), Self::Hash(_))
                | (Self::LBrace(_), Self::LBrace(_))
                | (Self::RBrace(_), Self::RBrace(_))
                | (Self::LBracket(_), Self::LBracket(_))
                | (Self::RBracket(_), Self::RBracket(_))
                | (Self::LParen(_), Self::LParen(_))
                | (Self::RParen(_), Self::RParen(_))
                | (Self::LAngle(_), Self::LAngle(_))
                | (Self::RAngle(_), Self::RAngle(_))
                | (Self::Colon(_), Self::Colon(_))
                | (Self::Semicolon(_), Self::Semicolon(_))
                | (Self::Comma(_), Self::Comma(_))
                | (Self::Equals(_), Self::Equals(_))
                | (Self::Unknown(_), Self::Unknown(_))
        )
    }

    pub const fn span(&self) -> Span {
        match self {
            Token::Ident(span)
            | Token::String(span)
            | Token::Number(span)
            | Token::Documentation(span)
            | Token::Hash(span)
            | Token::LBrace(span)
            | Token::RBrace(span)
            | Token::LBracket(span)
            | Token::RBracket(span)
            | Token::LParen(span)
            | Token::RParen(span)
            | Token::LAngle(span)
            | Token::RAngle(span)
            | Token::Colon(span)
            | Token::Semicolon(span)
            | Token::Comma(span)
            | Token::Equals(span)
            | Token::Unknown((_, span)) => *span,
        }
    }
}

pub struct Tokenizer<'src> {
    chars: Chars<'src>,
    offset: usize,
    current_char: Option<char>,
}

impl<'src> Tokenizer<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            chars: source.chars(),
            offset: 0,
            current_char: None,
        }
    }

    const fn create_span_with_length(&mut self, length: usize) -> Span {
        let span = Span {
            offset: self.offset,
            length,
        };
        self.offset += length;
        span
    }

    fn parse_ident(&mut self) -> Token {
        let mut length = 1;
        for c in self.chars.by_ref() {
            if c.is_alphanumeric() || c == '_' {
                length += 1;
            } else {
                self.current_char = Some(c);
                break;
            }
        }

        Token::Ident(self.create_span_with_length(length))
    }

    fn parse_string(&mut self) -> Token {
        self.offset += 1;
        let mut length = 0;
        for c in self.chars.by_ref() {
            if c == '"' {
                break;
            }
            length += 1;
        }
        let span = self.create_span_with_length(length);
        self.offset += 1;
        Token::String(span)
    }

    fn parse_number(&mut self) -> Token {
        let mut length = 1;

        for c in self.chars.by_ref() {
            if c.is_numeric() {
                length += 1;
            } else {
                self.current_char = Some(c);
                break;
            }
        }

        Token::Number(self.create_span_with_length(length))
    }

    fn parse_comment_or_documentation(&mut self) -> Option<Token> {
        self.offset += 1;
        let char = self.chars.next();

        match char {
            Some('/') => match self.chars.next() {
                Some('/') => {
                    self.offset += 2;
                    let mut length = 0;
                    for c in self.chars.by_ref() {
                        if c == '\n' {
                            break;
                        }

                        length += 1;
                    }
                    let span = self.create_span_with_length(length);
                    self.offset += 1;

                    Some(Token::Documentation(span))
                }
                Some(_) => {
                    self.offset += 2;
                    for c in self.chars.by_ref() {
                        self.offset += 1;
                        if c == '\n' {
                            break;
                        }
                    }

                    None
                }
                None => None,
            },
            Some(o) => Some(Token::Unknown((o, self.create_span_with_length(1)))),
            None => None,
        }
    }
}

impl Iterator for Tokenizer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(char) = self.current_char.take().or_else(|| self.chars.next()) {
            return Some(match char {
                c if c.is_whitespace() => {
                    self.offset += 1;
                    continue;
                }
                '#' => Token::Hash(self.create_span_with_length(1)),
                '{' => Token::LBrace(self.create_span_with_length(1)),
                '}' => Token::RBrace(self.create_span_with_length(1)),
                '[' => Token::LBracket(self.create_span_with_length(1)),
                ']' => Token::RBracket(self.create_span_with_length(1)),
                '(' => Token::LParen(self.create_span_with_length(1)),
                ')' => Token::RParen(self.create_span_with_length(1)),
                '<' => Token::LAngle(self.create_span_with_length(1)),
                '>' => Token::RAngle(self.create_span_with_length(1)),
                ':' => Token::Colon(self.create_span_with_length(1)),
                ';' => Token::Semicolon(self.create_span_with_length(1)),
                ',' => Token::Comma(self.create_span_with_length(1)),
                '=' => Token::Equals(self.create_span_with_length(1)),
                '_' => self.parse_ident(),
                '"' => self.parse_string(),
                '/' => {
                    let Some(t) = self.parse_comment_or_documentation() else {
                        continue;
                    };
                    t
                }
                c if c.is_alphabetic() => self.parse_ident(),
                c if c.is_numeric() || c == '-' => self.parse_number(),
                o => Token::Unknown((o, self.create_span_with_length(1))),
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::Tokenizer;
    use crate::tokenizer::Token;

    const SAMPLE: &str = include_str!("../../../system/system.cmp");
    #[test]
    fn test() {
        for token in Tokenizer::new(SAMPLE) {
            match token {
                Token::Ident(span) => {
                    let slice = &SAMPLE[span.as_range()];
                    println!("Ident: {slice}");
                }
                Token::String(span) => {
                    let slice = &SAMPLE[span.as_range()];
                    println!("String: {slice}");
                }
                Token::Number(span) => {
                    let slice = &SAMPLE[span.as_range()];
                    println!("Number: {slice}");
                }
                Token::Documentation(span) => {
                    let slice = &SAMPLE[span.as_range()];
                    println!("Documentation: {slice}");
                }
                o => {
                    println!("{o:#?}");
                }
            }
        }
    }
}
