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
            if byte == b' ' || byte == b'\t' {
                self.offset += 1;
                continue;
            }
            if byte == b'\n' {
                self.line += 1;

                continue;
            }
            //skip comments
            if byte == b'/' {
                if let Some(Ok(b'/')) = iterator.peek() {
                    while let Some(Ok(byte)) = iterator.next()
                        && byte != b'\n'
                    {}
                    continue;
                }
            }
            let token = match byte {
                b'{' => Token {
                    kind: TokenKind::CurlyBraceOpen,
                    span: Span {
                        start: self.offset,
                        end: self.offset + 1,
                        line: self.line,
                    },
                },
                b'}' => Token {
                    kind: TokenKind::CurlyBraceClose,
                    span: Span {
                        start: self.offset,
                        end: self.offset + 1,
                        line: self.line,
                    },
                },
                b'(' => Token {
                    kind: TokenKind::ParenOpen,
                    span: Span {
                        start: self.offset,
                        end: self.offset + 1,
                        line: self.line,
                    },
                },
                b')' => Token {
                    kind: TokenKind::ParenClose,
                    span: Span {
                        start: self.offset,
                        end: self.offset + 1,
                        line: self.line,
                    },
                },
                b';' => Token {
                    kind: TokenKind::SemiColon,
                    span: Span {
                        start: self.offset,
                        end: self.offset + 1,
                        line: self.line,
                    },
                },
                b'-' => Token {
                    kind: TokenKind::Negation,
                    span: Span {
                        start: self.offset,
                        end: self.offset + 1,
                        line: self.line,
                    },
                },
                b'~' => Token {
                    kind: TokenKind::BitComplement,
                    span: Span {
                        start: self.offset,
                        end: self.offset + 1,
                        line: self.line,
                    },
                },
                b'+' => Token {
                    kind: TokenKind::Add,
                    span: Span {
                        start: self.offset,
                        end: self.offset + 1,
                        line: self.line,
                    },
                },
                b'*' => Token {
                    kind: TokenKind::Mul,
                    span: Span {
                        start: self.offset,
                        end: self.offset + 1,
                        line: self.line,
                    },
                },
                b'/' => Token {
                    kind: TokenKind::Div,
                    span: Span {
                        start: self.offset,
                        end: self.offset + 1,
                        line: self.line,
                    },
                },
                b'%' => Token {
                    kind: TokenKind::Modulo,
                    span: Span {
                        start: self.offset,
                        end: self.offset + 1,
                        line: self.line,
                    },
                },
                b'^' => Token {
                    kind: TokenKind::BitwiseXor,
                    span: Span {
                        start: self.offset,
                        end: self.offset + 1,
                        line: self.line,
                    },
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
                        span: Span {
                            start: self.offset,
                            end: self.offset + 1,
                            line: self.line,
                        },
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
                        span: Span {
                            start: self.offset,
                            end: self.offset + 1,
                            line: self.line,
                        },
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
                        span: Span {
                            start: self.offset,
                            end: self.offset + 1,
                            line: self.line,
                        },
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
                        span: Span {
                            start: self.offset,
                            end: self.offset + 1,
                            line: self.line,
                        },
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
                        span: Span {
                            start: self.offset,
                            end: self.offset + 1,
                            line: self.line,
                        },
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
                        span: Span {
                            start: self.offset,
                            end: self.offset + 1,
                            line: self.line,
                        },
                    },
                },
                n if is_num(n) => self.lex_num(n, &mut iterator),
                a if is_alpha(a) => self.lex_alpha(a, &mut iterator),
                u => Token {
                    kind: TokenKind::Unknown(String::from_utf8_lossy(&[u]).to_string()),
                    span: Span {
                        start: self.offset,
                        end: self.offset + 1,
                        line: self.line,
                    },
                },
            };
            self.offset += 1;
            tokens.push(token);
        }
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
        self.offset += offset;
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
        let token = match string.as_ref() {
            "return" => Token {
                kind: TokenKind::Return,
                span: Span {
                    start: self.offset,
                    end: self.offset + offset,
                    line: self.line,
                },
            },
            "int" => Token {
                kind: TokenKind::Int,
                span: Span {
                    start: self.offset,
                    end: self.offset + offset,
                    line: self.line,
                },
            },
            _ => Token {
                kind: TokenKind::Ident(string),
                span: Span {
                    start: self.offset,
                    end: self.offset + offset,
                    line: self.line,
                },
            },
        };
        self.offset += offset;
        token
    }
}

fn is_num(byte: u8) -> bool {
    b'0' <= byte && byte <= b'9'
}

fn is_alpha(byte: u8) -> bool {
    b'a' <= byte && byte <= b'z' || b'A' <= byte && byte <= b'Z'
}
