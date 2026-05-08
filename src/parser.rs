#![allow(unused)]

use crate::lexer::Token;
use std::{fmt::Display, iter::Peekable};

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
    Unary {
        op: UnaryOp,
        expr: Box<Expression>,
    },
    Binary {
        op: BinaryOp,
        l_expr: Box<Expression>,
        r_expr: Box<Expression>,
    },
}

#[derive(Debug)]
pub enum UnaryOp {
    Negation,
    LogicalNegation,
    BitwiseComplement,
}
impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOp::Negation => write!(f, "-"),
            UnaryOp::LogicalNegation => write!(f, "!"),
            UnaryOp::BitwiseComplement => write!(f, "~"),
        }
    }
}

#[derive(Debug)]
pub enum BinaryOp {
    Add,
    Mul,
    Div,
    Sub,
    And,
    Or,
    Equal,
    NotEqual,
    LT,
    LTE,
    GT,
    GTE,
}
impl Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Mul => write!(f, "*"),
            BinaryOp::Div => write!(f, "/"),
            BinaryOp::Sub => write!(f, "-"),
            BinaryOp::And => write!(f, "&&"),
            BinaryOp::Or => write!(f, "||"),
            BinaryOp::Equal => write!(f, "=="),
            BinaryOp::NotEqual => write!(f, "!="),
            BinaryOp::LT => write!(f, "<"),
            BinaryOp::LTE => write!(f, "<="),
            BinaryOp::GT => write!(f, ">"),
            BinaryOp::GTE => write!(f, ">="),
        }
    }
}

impl Expression {
    fn fmt_tree(&self, f: &mut std::fmt::Formatter<'_>, depth: usize) -> std::fmt::Result {
        let indent = "\t".repeat(depth);
        match self {
            Expression::Const(n) => writeln!(f, "{indent}{n}"),

            Expression::Unary { op, expr } => {
                writeln!(f, "{indent}Unary ({op}):")?;
                expr.fmt_tree(f, depth + 1)
            }
            Expression::Binary { op, l_expr, r_expr } => {
                writeln!(f, "{indent}Binary ({op}):")?;
                writeln!(f, "{indent}\tLHS:")?;
                l_expr.fmt_tree(f, depth + 2)?;
                writeln!(f, "{indent}\tRHS:")?;
                r_expr.fmt_tree(f, depth + 2)
            }
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_tree(f, 0)
    }
}

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
// <equality-exp> ::= <relational-exp> { ("!=" | "==") <relational-exp> }
// <relational-exp> ::= <additive-exp> { ("<" | ">" | "<=" | ">=") <additive-exp> }
// <additive-exp> ::= <term> { ("+" | "-") <term> }
// <term> ::= <factor> { ("*" | "/") <factor> }
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
    let mut equality_expression = parse_equality(iterator);
    while let Some(next) = iterator.peek().cloned() {
        match next {
            Token::And => {
                iterator.next(); // eat &&
                equality_expression = Expression::Binary {
                    op: BinaryOp::And,
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
    let mut factor = parse_term(iterator);
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
                factor = Expression::Binary {
                    op,
                    l_expr: Box::new(factor),
                    r_expr: Box::new(parse_term(iterator)),
                }
            }
            _ => break,
        }
    }
    factor
}
fn parse_term(iterator: &mut Peekable<impl Iterator<Item = Token>>) -> Expression {
    let mut factor = parse_factor(iterator);
    while let Some(next) = iterator.peek().cloned() {
        match next {
            Token::Add | Token::Negation => {
                iterator.next(); // eat * or /
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
            Token::Mul | Token::Div => {
                iterator.next(); // eat + or -
                let op = if next == Token::Mul {
                    BinaryOp::Mul
                } else {
                    BinaryOp::Div
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
