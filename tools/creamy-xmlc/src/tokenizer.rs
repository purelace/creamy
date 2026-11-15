use std::str::Chars;

use miette::SourceSpan;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

#[derive(Debug)]
pub enum Token {
    Ident(Span),
    /// "..."
    String(Span),
    Number(Span),
    /// ///
    Documentation(Span),
    /// #
    Hash,
    /// {
    LBrace,
    /// }
    RBrace,
    /// [
    LBracket,
    /// ]
    RBracket,
    /// (
    LParen,
    /// )
    RParen,
    /// <
    LAngle,
    /// \>
    RAngle,
    /// :
    Colon,
    /// ;
    Semicolon,
    /// ,
    Comma,
    /// =
    Equals,
    Unknown(char),
}

impl Token {
    pub const fn matches(&self, rhs: &Self) -> bool {
        matches!(
            (self, rhs),
            (Self::Ident(_), Self::Ident(_))
                | (Self::String(_), Self::String(_))
                | (Self::Number(_), Self::Number(_))
                | (Self::Documentation(_), Self::Documentation(_))
                | (Self::Hash, Self::Hash)
                | (Self::LBrace, Self::LBrace)
                | (Self::RBrace, Self::RBrace)
                | (Self::LBracket, Self::LBracket)
                | (Self::RBracket, Self::RBracket)
                | (Self::LParen, Self::LParen)
                | (Self::RParen, Self::RParen)
                | (Self::LAngle, Self::LAngle)
                | (Self::RAngle, Self::RAngle)
                | (Self::Colon, Self::Colon)
                | (Self::Semicolon, Self::Semicolon)
                | (Self::Comma, Self::Comma)
                | (Self::Equals, Self::Equals)
                | (Self::Unknown(_), Self::Unknown(_))
        )
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
        self.offset += length + 1;
        span
    }

    fn parse_ident(&mut self) -> Token {
        self.offset -= 1;

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
        let mut length = 0;
        for c in self.chars.by_ref() {
            if c == '"' {
                break;
            }
            length += 1;
        }

        Token::String(self.create_span_with_length(length))
    }

    fn parse_number(&mut self) -> Token {
        self.offset -= 1;
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
                    self.offset += 1;
                    let mut length = 0;
                    for c in self.chars.by_ref() {
                        if c == '\n' {
                            break;
                        }

                        length += 1;
                    }

                    Some(Token::Documentation(self.create_span_with_length(length)))
                }
                Some(_) => {
                    self.offset += 1;
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
            Some(o) => Some(Token::Unknown(o)),
            None => None,
        }
    }
}

impl Iterator for Tokenizer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(char) = self.current_char.take().or_else(|| {
            self.offset += 1;
            self.chars.next()
        }) {
            return Some(match char {
                c if c.is_whitespace() => continue,
                '#' => Token::Hash,
                '{' => Token::LBrace,
                '}' => Token::RBrace,
                '[' => Token::LBracket,
                ']' => Token::RBracket,
                '(' => Token::LParen,
                ')' => Token::RParen,
                '<' => Token::LAngle,
                '>' => Token::RAngle,
                ':' => Token::Colon,
                ';' => Token::Semicolon,
                ',' => Token::Comma,
                '=' => Token::Equals,
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
                o => Token::Unknown(o),
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
