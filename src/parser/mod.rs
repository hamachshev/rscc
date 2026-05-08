#![allow(unused)]

use crate::lexer::types::Token;
use std::{fmt::Display, iter::Peekable};

pub mod types;
use types::{BinaryOp, Expression, Function, Program, Statement, Statements, UnaryOp};

pub fn parse_program(iterator: &mut Peekable<impl Iterator<Item = Token>>) -> Program {
    Program(parse_function(iterator))
}
fn parse_function(iterator: &mut Peekable<impl Iterator<Item = Token>>) -> Function {
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
fn parse_statement(iterator: &mut Peekable<impl Iterator<Item = Token>>) -> Statement {
    match get_token(iterator) {
        Token::Return => {
            let expr = parse_expression(iterator);
            expect(iterator, Token::SemiColon);
            Statement::Return(expr)
        }
        e => panic!("unexpected token, {:?}", e),
    }
}

// <exp> ::= <and-exp> { "||" <and-exp> }
// <and-exp> ::= <equality-exp> { "&&" <equality-exp> }
// <biwise-or-exp> ::= <bitwise-xor-expr> { "|" <bitwise-xor-expr> }
// <bitwise-xor-expr> ::= <bitwise-and-exp> { "^" <bitwise-and-exp> }
// <bitwise-and-expr>::= <equality-exp> { "&" <equality-exp> }
// <equality-exp> ::= <relational-exp> { ("!=" | "==") <relational-exp> }
// <relational-exp> ::= <bitwise-shift-exp> { ("<" | ">" | "<=" | ">=") <bitwise-shift-exp> }
// <bitwise-shift-exp> ::= <additive-exp> { ("<<" | ">>" <additive-exp>)}
// <additive-exp> ::= <term> { ("+" | "-") <term> }
// <term> ::= <factor> { ("*" | "/" | "%") <factor> }
// <factor> ::= "(" <exp> ")" | <unary_op> <factor> | <int>
// <unary_op> ::= "!" | "~" | "-"

fn parse_expression(iterator: &mut Peekable<impl Iterator<Item = Token>>) -> Expression {
    let mut and_exrp = parse_logical_and(iterator);
    while let Some(next) = iterator.peek().cloned() {
        match next {
            Token::Or => {
                iterator.next(); // eat ||
                and_exrp = Expression::Binary {
                    op: BinaryOp::Or,
                    l_expr: Box::new(and_exrp),
                    r_expr: Box::new(parse_logical_and(iterator)),
                }
            }
            _ => break,
        }
    }
    and_exrp
}

fn parse_logical_and(iterator: &mut Peekable<impl Iterator<Item = Token>>) -> Expression {
    let mut bitwise_or_expr = parse_bitwise_or(iterator);
    while let Some(next) = iterator.peek().cloned() {
        match next {
            Token::And => {
                iterator.next(); // eat &&
                bitwise_or_expr = Expression::Binary {
                    op: BinaryOp::And,
                    l_expr: Box::new(bitwise_or_expr),
                    r_expr: Box::new(parse_bitwise_or(iterator)),
                }
            }
            _ => break,
        }
    }
    bitwise_or_expr
}
fn parse_bitwise_or(iterator: &mut Peekable<impl Iterator<Item = Token>>) -> Expression {
    let mut bitwise_xor_expr = parse_bitwise_xor(iterator);
    while let Some(next) = iterator.peek().cloned() {
        match next {
            Token::BitwiseOr => {
                iterator.next(); // eat &&
                bitwise_xor_expr = Expression::Binary {
                    op: BinaryOp::BitwiseOr,
                    l_expr: Box::new(bitwise_xor_expr),
                    r_expr: Box::new(parse_bitwise_xor(iterator)),
                }
            }
            _ => break,
        }
    }
    bitwise_xor_expr
}
fn parse_bitwise_xor(iterator: &mut Peekable<impl Iterator<Item = Token>>) -> Expression {
    let mut bitwise_and_expr = parse_bitwise_and(iterator);
    while let Some(next) = iterator.peek().cloned() {
        match next {
            Token::BitwiseXor => {
                iterator.next(); // eat &&
                bitwise_and_expr = Expression::Binary {
                    op: BinaryOp::BitwiseXor,
                    l_expr: Box::new(bitwise_and_expr),
                    r_expr: Box::new(parse_bitwise_and(iterator)),
                }
            }
            _ => break,
        }
    }
    bitwise_and_expr
}
fn parse_bitwise_and(iterator: &mut Peekable<impl Iterator<Item = Token>>) -> Expression {
    let mut equality_expression = parse_equality(iterator);
    while let Some(next) = iterator.peek().cloned() {
        match next {
            Token::BitwiseAnd => {
                iterator.next(); // eat &&
                equality_expression = Expression::Binary {
                    op: BinaryOp::BitwiseAnd,
                    l_expr: Box::new(equality_expression),
                    r_expr: Box::new(parse_equality(iterator)),
                }
            }
            _ => break,
        }
    }
    equality_expression
}

// == !=
fn parse_equality(iterator: &mut Peekable<impl Iterator<Item = Token>>) -> Expression {
    let mut relational = parse_relational(iterator);
    while let Some(next) = iterator.peek().cloned() {
        match next {
            Token::Equal | Token::NotEqual => {
                iterator.next(); // eat == or !=
                let op = if next == Token::Equal {
                    BinaryOp::Equal
                } else {
                    BinaryOp::NotEqual
                };
                relational = Expression::Binary {
                    op,
                    l_expr: Box::new(relational),
                    r_expr: Box::new(parse_relational(iterator)),
                }
            }
            _ => break,
        }
    }
    relational
}

// < > <= >=
fn parse_relational(iterator: &mut Peekable<impl Iterator<Item = Token>>) -> Expression {
    let mut bitwise_shift_expr = parse_bitwise_shift(iterator);
    while let Some(next) = iterator.peek().cloned() {
        match next {
            Token::LT | Token::LTE | Token::GT | Token::GTE => {
                iterator.next(); // eat < > <= >=
                let op = match next {
                    Token::LT => BinaryOp::LT,
                    Token::LTE => BinaryOp::LTE,
                    Token::GT => BinaryOp::GT,
                    Token::GTE => BinaryOp::GTE,
                    _ => panic!("unreachable"),
                };
                bitwise_shift_expr = Expression::Binary {
                    op,
                    l_expr: Box::new(bitwise_shift_expr),
                    r_expr: Box::new(parse_bitwise_shift(iterator)),
                }
            }
            _ => break,
        }
    }
    bitwise_shift_expr
}

fn parse_bitwise_shift(iterator: &mut Peekable<impl Iterator<Item = Token>>) -> Expression {
    let mut term = parse_term(iterator);
    while let Some(next) = iterator.peek().cloned() {
        match next {
            Token::BitwiseShiftLeft | Token::BitwiseShiftRight => {
                iterator.next(); // eat << or >>
                let op = if next == Token::BitwiseShiftLeft {
                    BinaryOp::BitwiseShiftLeft
                } else {
                    BinaryOp::BitwiseShiftRight
                };
                term = Expression::Binary {
                    op,
                    l_expr: Box::new(term),
                    r_expr: Box::new(parse_term(iterator)),
                }
            }
            _ => break,
        }
    }
    term
}
fn parse_term(iterator: &mut Peekable<impl Iterator<Item = Token>>) -> Expression {
    let mut factor = parse_factor(iterator);
    while let Some(next) = iterator.peek().cloned() {
        match next {
            Token::Add | Token::Negation => {
                iterator.next(); // eat + or -
                let op = if next == Token::Add {
                    BinaryOp::Add
                } else {
                    BinaryOp::Sub
                };
                factor = Expression::Binary {
                    op,
                    l_expr: Box::new(factor),
                    r_expr: Box::new(parse_factor(iterator)),
                }
            }
            _ => break,
        }
    }
    factor
}

fn parse_factor(iterator: &mut Peekable<impl Iterator<Item = Token>>) -> Expression {
    let mut factor = parse_unary(iterator);
    while let Some(next) = iterator.peek().cloned() {
        match next {
            Token::Mul | Token::Div | Token::Modulo => {
                iterator.next(); // eat * or /
                let op = match next {
                    Token::Mul => BinaryOp::Mul,
                    Token::Div => BinaryOp::Div,
                    Token::Modulo => BinaryOp::Modulo,
                    _ => panic!("should never happen"),
                };
                factor = Expression::Binary {
                    op,
                    l_expr: Box::new(factor),
                    r_expr: Box::new(parse_unary(iterator)),
                }
            }
            _ => break,
        }
    }
    factor
}

fn parse_unary(iterator: &mut Peekable<impl Iterator<Item = Token>>) -> Expression {
    match iterator.next() {
        Some(token) => match token {
            Token::Integer(i) => Expression::Const(i),
            Token::Negation => Expression::Unary {
                op: UnaryOp::Negation,
                expr: Box::new(parse_factor(iterator)),
            },
            Token::BitComplement => Expression::Unary {
                op: UnaryOp::BitwiseComplement,
                expr: Box::new(parse_factor(iterator)),
            },
            Token::LogicalNegation => Expression::Unary {
                op: UnaryOp::LogicalNegation,
                expr: Box::new(parse_factor(iterator)),
            },
            Token::ParenOpen => {
                let expr = parse_expression(iterator);
                expect(iterator, Token::ParenClose);
                expr
            }
            t => panic!("expected expression, got {:?}", t),
        },
        None => panic!("unexpected EOF"),
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
