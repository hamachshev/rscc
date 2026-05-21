use std::{
    io::{self, Bytes, Read},
    iter::Peekable,
};

pub mod types;
pub use crate::lexer::types::{Lexer, Span};
use types::{Token, TokenKind};

impl Lexer {
    pub fn new() -> Self {
        Lexer { line: 0, offset: 0 }
    }
    pub fn lex(&mut self, source: impl Read) -> Result<Vec<Token>, io::Error> {
        let mut tokens = Vec::new();
        let mut iterator = source.bytes().peekable();

        while let Some(Ok(byte)) = iterator.next() {
            //skip whitespace
            if byte == b' ' {
                self.offset += 1;
                continue;
            }
            if byte == b'\t' {
                self.offset += 1; // tabs are one char offset, we calc tabs later in error msg for
                // correct formatting
                continue;
            }
            if byte == b'\n' {
                self.line += 1;
                self.offset += 1;

                continue;
            }
            //skip comments
            if byte == b'/' {
                if let Some(Ok(b'/')) = iterator.peek() {
                    while let Some(Ok(byte)) = iterator.next()
                        && byte != b'\n'
                    {
                        // FIXME: this is wrong
                        self.offset += 2;
                    }
                    continue;
                }
            }
            let single_token_span = Span {
                start: self.offset,
                end: self.offset + 1,
                line: self.line,
            };

            let token = match byte {
                b'{' => Token {
                    kind: TokenKind::CurlyBraceOpen,
                    span: single_token_span,
                },
                b'}' => Token {
                    kind: TokenKind::CurlyBraceClose,
                    span: single_token_span,
                },
                b'(' => Token {
                    kind: TokenKind::ParenOpen,
                    span: single_token_span,
                },
                b')' => Token {
                    kind: TokenKind::ParenClose,
                    span: single_token_span,
                },
                b';' => Token {
                    kind: TokenKind::SemiColon,
                    span: single_token_span,
                },
                b'-' => Token {
                    kind: TokenKind::Negation,
                    span: single_token_span,
                },
                b'~' => Token {
                    kind: TokenKind::BitComplement,
                    span: single_token_span,
                },
                b'+' => Token {
                    kind: TokenKind::Add,
                    span: single_token_span,
                },
                b'*' => Token {
                    kind: TokenKind::Mul,
                    span: single_token_span,
                },
                b'/' => Token {
                    kind: TokenKind::Div,
                    span: single_token_span,
                },
                b'%' => Token {
                    kind: TokenKind::Modulo,
                    span: single_token_span,
                },
                b'^' => Token {
                    kind: TokenKind::BitwiseXor,
                    span: single_token_span,
                },
                b':' => Token {
                    kind: TokenKind::Colon,
                    span: single_token_span,
                },
                b'?' => Token {
                    kind: TokenKind::QMark,
                    span: single_token_span,
                },
                b'&' => match iterator.peek() {
                    Some(Ok(b'&')) => {
                        iterator.next();
                        self.offset += 1;
                        Token {
                            kind: TokenKind::And,
                            span: Span {
                                start: self.offset - 1,
                                end: self.offset + 1,
                                line: self.line,
                            },
                        }
                    }
                    _ => Token {
                        kind: TokenKind::BitwiseAnd,
                        span: single_token_span,
                    },
                },
                b'|' => match iterator.peek() {
                    Some(Ok(b'|')) => {
                        iterator.next();
                        self.offset += 1;
                        Token {
                            kind: TokenKind::Or,
                            span: Span {
                                start: self.offset - 1,
                                end: self.offset + 1,
                                line: self.line,
                            },
                        }
                    }
                    _ => Token {
                        kind: TokenKind::BitwiseOr,
                        span: single_token_span,
                    },
                },

                b'=' => match iterator.peek() {
                    Some(Ok(b'=')) => {
                        iterator.next();
                        self.offset += 1;
                        Token {
                            kind: TokenKind::Equal,
                            span: Span {
                                start: self.offset - 1,
                                end: self.offset + 1,
                                line: self.line,
                            },
                        }
                    }
                    _ => Token {
                        kind: TokenKind::Assign,
                        span: single_token_span,
                    },
                },
                b'!' => match iterator.peek() {
                    Some(Ok(b'=')) => {
                        iterator.next();
                        self.offset += 1;
                        Token {
                            kind: TokenKind::NotEqual,
                            span: Span {
                                start: self.offset - 1,
                                end: self.offset + 1,
                                line: self.line,
                            },
                        }
                    }
                    _ => Token {
                        kind: TokenKind::LogicalNegation,
                        span: single_token_span,
                    },
                },
                b'<' => match iterator.peek() {
                    Some(Ok(b'=')) => {
                        iterator.next();
                        self.offset += 1;
                        Token {
                            kind: TokenKind::LTE,
                            span: Span {
                                start: self.offset - 1,
                                end: self.offset + 1,
                                line: self.line,
                            },
                        }
                    }
                    Some(Ok(b'<')) => {
                        iterator.next();
                        self.offset += 1;
                        Token {
                            kind: TokenKind::BitwiseShiftLeft,
                            span: Span {
                                start: self.offset - 1,
                                end: self.offset + 1,
                                line: self.line,
                            },
                        }
                    }
                    _ => Token {
                        kind: TokenKind::LT,
                        span: single_token_span,
                    },
                },
                b'>' => match iterator.peek() {
                    Some(Ok(b'=')) => {
                        iterator.next();
                        self.offset += 1;
                        Token {
                            kind: TokenKind::GTE,
                            span: Span {
                                start: self.offset - 1,
                                end: self.offset + 1,
                                line: self.line,
                            },
                        }
                    }
                    Some(Ok(b'>')) => {
                        iterator.next();
                        self.offset += 1;
                        Token {
                            kind: TokenKind::BitwiseShiftRight,
                            span: Span {
                                start: self.offset - 1,
                                end: self.offset + 1,
                                line: self.line,
                            },
                        }
                    }
                    _ => Token {
                        kind: TokenKind::GT,
                        span: single_token_span,
                    },
                },
                n if is_num(n) => self.lex_num(n, &mut iterator),
                a if is_alpha(a) => self.lex_alpha(a, &mut iterator),
                u => Token {
                    kind: TokenKind::Unknown(String::from_utf8_lossy(&[u]).to_string()),
                    span: single_token_span,
                },
            };
            self.offset += 1;
            tokens.push(token);
        }
        //add EOF - span is -1 bc added to offset above
        tokens.push(Token {
            kind: TokenKind::EOF,
            span: Span {
                start: self.offset - 1,
                end: self.offset - 1,
                line: self.line,
            },
        });
        Ok(tokens)
    }
    fn lex_num(&mut self, num_start: u8, iterator: &mut Peekable<Bytes<impl Read>>) -> Token {
        let mut num = vec![num_start];
        while let Some(Ok(byte)) = iterator.peek() {
            if is_num(*byte) {
                num.push(iterator.next().unwrap().unwrap());
            } else {
                break;
            }
        }
        let offset = num.len();

        let token = Token {
            kind: TokenKind::Integer(String::from_utf8(num).unwrap().parse::<usize>().unwrap()),
            span: Span {
                start: self.offset,
                end: self.offset + offset,
                line: self.line,
            },
        };
        self.offset += offset - 1; //we are going to add 1 at the end
        token
    }

    fn lex_alpha(&mut self, str_start: u8, iterator: &mut Peekable<Bytes<impl Read>>) -> Token {
        let mut string = vec![str_start];
        while let Some(Ok(byte)) = iterator.peek() {
            if is_alpha(*byte) || is_num(*byte) {
                string.push(iterator.next().unwrap().unwrap());
            } else {
                break;
            }
        }
        let offset = string.len();
        let string = String::from_utf8(string).unwrap();
        let span = Span {
            start: self.offset,
            end: self.offset + offset,
            line: self.line,
        };

        let token = match string.as_ref() {
            "return" => Token {
                kind: TokenKind::Return,
                span,
            },
            "int" => Token {
                kind: TokenKind::Int,
                span,
            },
            "if" => Token {
                kind: TokenKind::If,
                span,
            },
            "else" => Token {
                kind: TokenKind::Else,
                span,
            },
            _ => Token {
                kind: TokenKind::Ident(string),
                span,
            },
        };
        self.offset += offset - 1; //we are going to add 1 at the end
        token
    }
}

fn is_num(byte: u8) -> bool {
    b'0' <= byte && byte <= b'9'
}

fn is_alpha(byte: u8) -> bool {
    b'a' <= byte && byte <= b'z' || b'A' <= byte && byte <= b'Z'
}
