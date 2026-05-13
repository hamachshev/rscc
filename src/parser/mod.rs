#![allow(unused)]

use crate::lexer::types::{Token, TokenKind};
use std::{
    cmp,
    fmt::{Debug, Display},
    iter::Peekable,
    os, process,
};
use types::{BinaryOp, Expression, Function, Program, Statement, Statements, UnaryOp};

pub mod types;

pub struct Parser<'a> {
    buf: &'a [u8],
}
impl<'a> Parser<'a> {
    pub fn new(buf: &'a [u8]) -> Parser<'a> {
        Parser { buf }
    }
    pub fn parse_program(&self, iterator: &mut Peekable<impl Iterator<Item = Token>>) -> Program {
        Program(self.parse_function(iterator))
    }
    fn parse_function(&self, iterator: &mut Peekable<impl Iterator<Item = Token>>) -> Function {
        let return_type = match get_token(iterator) {
            Token {
                kind: TokenKind::Int,
                span: _,
            } => "",
            e => self.print_error(e, "return type"),
        };
        let ident = match get_token(iterator) {
            Token {
                kind: TokenKind::Ident(ident),
                span: _,
            } => ident,
            e => self.print_error(e, "ident"),
        };
        self.expect(iterator, TokenKind::ParenOpen);
        self.expect(iterator, TokenKind::ParenClose);
        self.expect(iterator, TokenKind::CurlyBraceOpen);
        let mut statements = Vec::new();
        loop {
            match iterator.peek() {
                Some(Token {
                    kind: TokenKind::CurlyBraceClose,
                    span: _,
                }) => break,
                Some(_) => statements.push(self.parse_statement(iterator)),
                None => panic!("unexpected EOF"),
            }
        }

        Function(ident, Statements(statements))
    }
    fn parse_statement(&self, iterator: &mut Peekable<impl Iterator<Item = Token>>) -> Statement {
        let Token {
            kind: token,
            span: _,
        } = get_peek(iterator);
        match token {
            TokenKind::Return => {
                iterator.next(); //eat return
                let expr = self.parse_expression(iterator);
                self.expect(iterator, TokenKind::SemiColon);
                Statement::Return(expr)
            }
            TokenKind::Int => {
                iterator.next(); //eat int
                let expr = self.parse_expression(iterator);
                match expr {
                    Expression::Assign { ident, expr } => {
                        self.expect(iterator, TokenKind::SemiColon);
                        Statement::Declare(*ident, Some(*expr))
                    }
                    Expression::Ident(_) => Statement::Declare(expr, None),
                    e => panic!("Expected expression, got {:?}", e),
                }
            }
            _ => {
                let expr = self.parse_expression(iterator);
                self.expect(iterator, TokenKind::SemiColon);
                Statement::Expr(expr)
            }
        }
    }
    // <program> ::= <function>
    // <function> ::= "int" <id> "(" ")" "{" { <statement> } "}"
    // <statement> ::= "return" <exp> ";"
    //               | <exp> ";"
    //               | "int" <ident> [ = <exp>] ";"
    // <exp> ::= <ident> "=" <exp> | <logical-or-exp>
    // <logical_or_exp> ::= <and-exp> { "||" <and-exp> }
    // <and-exp> ::= <equality-exp> { "&&" <equality-exp> }
    // <biwise-or-exp> ::= <bitwise-xor-expr> { "|" <bitwise-xor-expr> }
    // <bitwise-xor-expr> ::= <bitwise-and-exp> { "^" <bitwise-and-exp> }
    //; <bitwise-and-expr>::= <equality-exp> { "&" <equality-exp> }
    // <equality-exp> ::= <relational-exp> { ("!=" | "==") <relational-exp> }
    // <relational-exp> ::= <bitwise-shift-exp> { ("<" | ">" | "<=" | ">=") <bitwise-shift-exp> }
    // <bitwise-shift-exp> ::= <additive-exp> { ("<<" | ">>" <additive-exp>)}
    // <additive-exp> ::= <term> { ("+" | "-") <term> }
    // <term> ::= <factor> { ("*" | "/" | "%") <factor> }
    // <factor> ::= "(" <exp> ")" | <unary_op> <factor> | <int> | <ident>
    // <unary_op> ::= "!" | "~" | "-"

    fn parse_expression(&self, iterator: &mut Peekable<impl Iterator<Item = Token>>) -> Expression {
        match iterator.peek() {
            Some(Token {
                kind: TokenKind::Ident(_),
                span: _,
            }) => self.parse_assignment(iterator),
            Some(_) => self.parse_logical_or(iterator),
            None => {
                panic!("unexpected EOF")
            }
        }
    }

    fn parse_assignment(&self, iterator: &mut Peekable<impl Iterator<Item = Token>>) -> Expression {
        let ident = self.parse_unary(iterator);
        let Expression::Ident(_) = ident else {
            panic!("expeceted ident, got {:?}", ident)
        };
        match iterator.peek() {
            Some(Token {
                kind: TokenKind::Assign,
                span: _,
            }) => {
                self.expect(iterator, TokenKind::Assign);
                let expr = self.parse_expression(iterator);

                Expression::Assign {
                    ident: Box::new(ident),
                    expr: Box::new(expr),
                }
            }
            Some(_) => ident,
            None => panic!("unexpected EOF"),
        }
    }
    fn parse_logical_or(&self, iterator: &mut Peekable<impl Iterator<Item = Token>>) -> Expression {
        let mut and_exrp = self.parse_logical_and(iterator);
        while let Some(Token {
            kind: next,
            span: _,
        }) = iterator.peek().cloned()
        {
            match next {
                TokenKind::Or => {
                    iterator.next(); // eat ||
                    and_exrp = Expression::Binary {
                        op: BinaryOp::Or,
                        l_expr: Box::new(and_exrp),
                        r_expr: Box::new(self.parse_logical_and(iterator)),
                    }
                }
                _ => break,
            }
        }
        and_exrp
    }

    fn parse_logical_and(
        &self,
        iterator: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> Expression {
        let mut bitwise_or_expr = self.parse_bitwise_or(iterator);
        while let Some(Token {
            kind: next,
            span: _,
        }) = iterator.peek().cloned()
        {
            match next {
                TokenKind::And => {
                    iterator.next(); // eat &&
                    bitwise_or_expr = Expression::Binary {
                        op: BinaryOp::And,
                        l_expr: Box::new(bitwise_or_expr),
                        r_expr: Box::new(self.parse_bitwise_or(iterator)),
                    }
                }
                _ => break,
            }
        }
        bitwise_or_expr
    }
    fn parse_bitwise_or(&self, iterator: &mut Peekable<impl Iterator<Item = Token>>) -> Expression {
        let mut bitwise_xor_expr = self.parse_bitwise_xor(iterator);
        while let Some(Token {
            kind: next,
            span: _,
        }) = iterator.peek().cloned()
        {
            match next {
                TokenKind::BitwiseOr => {
                    iterator.next(); // eat &&
                    bitwise_xor_expr = Expression::Binary {
                        op: BinaryOp::BitwiseOr,
                        l_expr: Box::new(bitwise_xor_expr),
                        r_expr: Box::new(self.parse_bitwise_xor(iterator)),
                    }
                }
                _ => break,
            }
        }
        bitwise_xor_expr
    }
    fn parse_bitwise_xor(
        &self,
        iterator: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> Expression {
        let mut bitwise_and_expr = self.parse_bitwise_and(iterator);
        while let Some(Token {
            kind: next,
            span: _,
        }) = iterator.peek().cloned()
        {
            match next {
                TokenKind::BitwiseXor => {
                    iterator.next(); // eat &&
                    bitwise_and_expr = Expression::Binary {
                        op: BinaryOp::BitwiseXor,
                        l_expr: Box::new(bitwise_and_expr),
                        r_expr: Box::new(self.parse_bitwise_and(iterator)),
                    }
                }
                _ => break,
            }
        }
        bitwise_and_expr
    }
    fn parse_bitwise_and(
        &self,
        iterator: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> Expression {
        let mut equality_expression = self.parse_equality(iterator);

        while let Some(Token {
            kind: next,
            span: _,
        }) = iterator.peek().cloned()
        {
            match next {
                TokenKind::BitwiseAnd => {
                    iterator.next(); // eat &&
                    equality_expression = Expression::Binary {
                        op: BinaryOp::BitwiseAnd,
                        l_expr: Box::new(equality_expression),
                        r_expr: Box::new(self.parse_equality(iterator)),
                    }
                }
                _ => break,
            }
        }
        equality_expression
    }

    // == !=
    fn parse_equality(&self, iterator: &mut Peekable<impl Iterator<Item = Token>>) -> Expression {
        let mut relational = self.parse_relational(iterator);

        while let Some(Token {
            kind: next,
            span: _,
        }) = iterator.peek().cloned()
        {
            match next {
                TokenKind::Equal | TokenKind::NotEqual => {
                    iterator.next(); // eat == or !=
                    let op = if next == TokenKind::Equal {
                        BinaryOp::Equal
                    } else {
                        BinaryOp::NotEqual
                    };
                    relational = Expression::Binary {
                        op,
                        l_expr: Box::new(relational),
                        r_expr: Box::new(self.parse_relational(iterator)),
                    }
                }
                _ => break,
            }
        }
        relational
    }

    // < > <= >=
    fn parse_relational(&self, iterator: &mut Peekable<impl Iterator<Item = Token>>) -> Expression {
        let mut bitwise_shift_expr = self.parse_bitwise_shift(iterator);
        while let Some(Token {
            kind: next,
            span: _,
        }) = iterator.peek().cloned()
        {
            match next {
                TokenKind::LT | TokenKind::LTE | TokenKind::GT | TokenKind::GTE => {
                    iterator.next(); // eat < > <= >=

                    let op = match next {
                        TokenKind::LT => BinaryOp::LT,
                        TokenKind::LTE => BinaryOp::LTE,
                        TokenKind::GT => BinaryOp::GT,
                        TokenKind::GTE => BinaryOp::GTE,
                        _ => panic!("unreachable"),
                    };
                    bitwise_shift_expr = Expression::Binary {
                        op,
                        l_expr: Box::new(bitwise_shift_expr),
                        r_expr: Box::new(self.parse_bitwise_shift(iterator)),
                    }
                }
                _ => break,
            }
        }
        bitwise_shift_expr
    }

    fn parse_bitwise_shift(
        &self,
        iterator: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> Expression {
        let mut term = self.parse_term(iterator);

        while let Some(Token {
            kind: next,
            span: _,
        }) = iterator.peek().cloned()
        {
            match next {
                TokenKind::BitwiseShiftLeft | TokenKind::BitwiseShiftRight => {
                    iterator.next(); // eat << or >>
                    let op = if next == TokenKind::BitwiseShiftLeft {
                        BinaryOp::BitwiseShiftLeft
                    } else {
                        BinaryOp::BitwiseShiftRight
                    };
                    term = Expression::Binary {
                        op,
                        l_expr: Box::new(term),
                        r_expr: Box::new(self.parse_term(iterator)),
                    }
                }
                _ => break,
            }
        }
        term
    }
    fn parse_term(&self, iterator: &mut Peekable<impl Iterator<Item = Token>>) -> Expression {
        let mut factor = self.parse_factor(iterator);

        while let Some(Token {
            kind: next,
            span: _,
        }) = iterator.peek().cloned()
        {
            match next {
                TokenKind::Add | TokenKind::Negation => {
                    iterator.next(); // eat + or -
                    let op = if next == TokenKind::Add {
                        BinaryOp::Add
                    } else {
                        BinaryOp::Sub
                    };
                    factor = Expression::Binary {
                        op,
                        l_expr: Box::new(factor),
                        r_expr: Box::new(self.parse_factor(iterator)),
                    }
                }
                _ => break,
            }
        }
        factor
    }

    fn parse_factor(&self, iterator: &mut Peekable<impl Iterator<Item = Token>>) -> Expression {
        let mut factor = self.parse_unary(iterator);
        while let Some(Token {
            kind: next,
            span: _,
        }) = iterator.peek().cloned()
        {
            match next {
                TokenKind::Mul | TokenKind::Div | TokenKind::Modulo => {
                    iterator.next(); // eat * or /
                    let op = match next {
                        TokenKind::Mul => BinaryOp::Mul,
                        TokenKind::Div => BinaryOp::Div,
                        TokenKind::Modulo => BinaryOp::Modulo,
                        _ => panic!("should never happen"),
                    };
                    factor = Expression::Binary {
                        op,
                        l_expr: Box::new(factor),
                        r_expr: Box::new(self.parse_unary(iterator)),
                    }
                }
                _ => break,
            }
        }
        factor
    }

    fn parse_unary(&self, iterator: &mut Peekable<impl Iterator<Item = Token>>) -> Expression {
        match iterator.next() {
            Some(token) => match token {
                Token {
                    kind: TokenKind::Ident(i),
                    span: _,
                } => Expression::Ident(i),
                Token {
                    kind: TokenKind::Integer(i),
                    span: _,
                } => Expression::Const(i),
                Token {
                    kind: TokenKind::Negation,
                    span: _,
                } => Expression::Unary {
                    op: UnaryOp::Negation,
                    expr: Box::new(self.parse_factor(iterator)),
                },
                Token {
                    kind: TokenKind::BitComplement,
                    span: _,
                } => Expression::Unary {
                    op: UnaryOp::BitwiseComplement,
                    expr: Box::new(self.parse_factor(iterator)),
                },
                Token {
                    kind: TokenKind::LogicalNegation,
                    span: _,
                } => Expression::Unary {
                    op: UnaryOp::LogicalNegation,
                    expr: Box::new(self.parse_factor(iterator)),
                },
                Token {
                    kind: TokenKind::ParenOpen,
                    span: _,
                } => {
                    let expr = self.parse_expression(iterator);
                    self.expect(iterator, TokenKind::ParenClose);
                    expr
                }
                t => self.print_error(t, "hello"),
            },
            None => panic!("unexpected EOF"),
        }
    }

    fn print_error(&self, token: Token, expected: &str) -> ! {
        let start = cmp::max(token.span.start as isize - 5, 0);
        let end = cmp::min(token.span.end + 5, self.buf.len());
        eprintln!("{}ERROR{}\n", "-".repeat(10), "-".repeat(10));
        let line_msg = format!("line {}: ", token.span.line + 1);
        eprintln!(
            "{}{}",
            line_msg,
            String::from_utf8_lossy(&self.buf[start as usize..end]).trim_end()
        );

        eprintln!(
            "{}^{}expected: {}, got: {:?}",
            " ".repeat(line_msg.len() + (token.span.start as isize - start) as usize),
            "-".repeat(20),
            expected,
            token.kind
        );
        eprintln!("\n{}", "-".repeat(20 + "error".len()));
        process::exit(1);
    }
    fn expect(&self, iterator: &mut impl Iterator<Item = Token>, expect: TokenKind) {
        match iterator.next() {
            Some(Token {
                kind: token,
                span: _,
            }) if token == expect => {}
            Some(token) => self.print_error(token, &format!("{:?}", expect)),
            None => panic!("unexpected EOF, expected {:?}", expect),
        }
    }
}

fn get_token(iterator: &mut impl Iterator<Item = Token>) -> Token {
    iterator.next().unwrap_or_else(|| panic!("unexpected EOF"))
}
fn get_peek(iterator: &mut Peekable<impl Iterator<Item = Token>>) -> &Token {
    iterator.peek().unwrap_or_else(|| panic!("unexpected EOF"))
}
