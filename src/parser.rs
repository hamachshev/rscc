#![allow(unused)]

use crate::lexer::Token;
use std::fmt::Display;

#[derive(Debug)]
pub enum Statement {
    Return(Expression),
}
impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Return(expression) => write!(f, "RETURN Int <{}>", expression),
        }
    }
}

#[derive(Debug)]
pub struct Statements(pub Vec<Statement>);

impl Display for Statements {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for statement in self.0.iter() {
            write!(f, "{}", statement)?
        }
        write!(f, "\n")
    }
}

#[derive(Debug)]
pub struct Function(pub String, pub Statements);

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "FUN INT {}:\n\tparams:()\n\tbody:\n\t\t{}",
            self.0, self.1
        )
    }
}

#[derive(Debug)]
pub struct Program(pub Function);

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub enum Expression {
    Const(usize),
}
impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Const(n) => write!(f, "{}", n),
        }
    }
}

pub fn parse_program(iterator: &mut impl Iterator<Item = Token>) -> Program {
    Program(parse_function(iterator))
}
fn parse_function(iterator: &mut impl Iterator<Item = Token>) -> Function {
    let return_type = match get_token(iterator) {
        Token::Int => "",
        e => panic!("missing return type"),
    };
    let ident = match get_token(iterator) {
        Token::Ident(ident) => ident,
        e => panic!("expected ident, got {:?}", e),
    };
    expect(iterator, Token::ParenOpen);
    expect(iterator, Token::ParenClose);
    expect(iterator, Token::CurlyBraceOpen);
    let mut statements = Vec::new();
    statements.push(parse_statement(iterator));
    expect(iterator, Token::CurlyBraceClose);

    Function(ident, Statements(statements))
}
fn parse_statement(iterator: &mut impl Iterator<Item = Token>) -> Statement {
    match get_token(iterator) {
        Token::Return => {
            let expr = match iterator.next() {
                Some(Token::Integer(i)) => Expression::Const(i),
                Some(t) => panic!("expected expression, got {:?}", t),
                None => panic!("unexpected EOF, expected expression"),
            };
            expect(iterator, Token::SemiColon);
            Statement::Return(expr)
        }
        e => panic!("unexpected token, {:?}", e),
    }
}

fn expect(iterator: &mut impl Iterator<Item = Token>, expect: Token) {
    match iterator.next() {
        Some(token) if token == expect => {}
        Some(token) => panic!("expected {:?}, got {:?}", expect, token),
        None => panic!("unexpected EOF, expected {:?}", expect),
    }
}

fn get_token(iterator: &mut impl Iterator<Item = Token>) -> Token {
    iterator.next().unwrap_or_else(|| panic!("unexpected EOF"))
}
