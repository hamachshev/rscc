use std::{
    io::{self, Bytes, Read},
    iter::Peekable,
};

pub mod types;
use types::Token;

#[allow(unused)]
pub fn lex(source: impl Read) -> Result<Vec<Token>, io::Error> {
    let mut tokens = Vec::new();
    let mut iterator = source.bytes().peekable();

    while let Some(Ok(byte)) = iterator.next() {
        //skip whitespace
        if byte == b' ' || byte == b'\t' || byte == b'\n' {
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
            b'{' => Token::CurlyBraceOpen,
            b'}' => Token::CurlyBraceClose,
            b'(' => Token::ParenOpen,
            b')' => Token::ParenClose,
            b';' => Token::SemiColon,
            b'-' => Token::Negation,
            b'~' => Token::BitComplement,
            b'+' => Token::Add,
            b'*' => Token::Mul,
            b'/' => Token::Div,
            b'%' => Token::Modulo,
            b'^' => Token::BitwiseXor,
            b'&' => match iterator.peek() {
                Some(Ok(b'&')) => {
                    iterator.next();
                    Token::And
                }
                _ => Token::BitwiseAnd,
            },
            b'|' => match iterator.peek() {
                Some(Ok(b'|')) => {
                    iterator.next();
                    Token::Or
                }
                _ => Token::BitwiseOr,
            },

            b'=' => match iterator.peek() {
                Some(Ok(b'=')) => {
                    iterator.next();
                    Token::Equal
                }
                _ => todo!(),
            },
            b'!' => match iterator.peek() {
                Some(Ok(b'=')) => {
                    iterator.next();
                    Token::NotEqual
                }
                _ => Token::LogicalNegation,
            },
            b'<' => match iterator.peek() {
                Some(Ok(b'=')) => {
                    iterator.next();
                    Token::LTE
                }
                Some(Ok(b'<')) => {
                    iterator.next();
                    Token::BitwiseShiftLeft
                }
                _ => Token::LT,
            },
            b'>' => match iterator.peek() {
                Some(Ok(b'=')) => {
                    iterator.next();
                    Token::GTE
                }
                Some(Ok(b'>')) => {
                    iterator.next();
                    Token::BitwiseShiftRight
                }
                _ => Token::GT,
            },
            n if is_num(n) => lex_num(n, &mut iterator),
            a if is_alpha(a) => lex_alpha(a, &mut iterator),
            u => Token::Unknown(String::from_utf8_lossy(&[u]).to_string()),
        };
        tokens.push(token);
    }
    Ok(tokens)
}

fn is_num(byte: u8) -> bool {
    b'0' <= byte && byte <= b'9'
}

fn lex_num(num_start: u8, iterator: &mut Peekable<Bytes<impl Read>>) -> Token {
    let mut num = vec![num_start];
    while let Some(Ok(byte)) = iterator.peek() {
        if is_num(*byte) {
            num.push(iterator.next().unwrap().unwrap());
        } else {
            break;
        }
    }
    Token::Integer(String::from_utf8(num).unwrap().parse::<usize>().unwrap())
}

fn is_alpha(byte: u8) -> bool {
    b'a' <= byte && byte <= b'z' || b'A' <= byte && byte <= b'Z'
}

fn lex_alpha(str_start: u8, iterator: &mut Peekable<Bytes<impl Read>>) -> Token {
    let mut string = vec![str_start];
    while let Some(Ok(byte)) = iterator.peek() {
        if is_alpha(*byte) || is_num(*byte) {
            string.push(iterator.next().unwrap().unwrap());
        } else {
            break;
        }
    }
    let string = String::from_utf8(string).unwrap();
    match string.as_ref() {
        "return" => Token::Return,
        "int" => Token::Int,
        _ => Token::Ident(string),
    }
}
