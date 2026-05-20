use std::fmt::Display;

use itertools::MultiPeek;

#[derive(Debug)]
pub struct Program(pub Function);

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct Function(pub String, pub Block);

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
pub struct Block(pub Vec<BlockItem>);

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for statement in self.0.iter() {
            write!(f, "{}", statement)?
        }
        write!(f, "\n")
    }
}

#[derive(Debug)]
pub enum BlockItem {
    Statement(Statement),
    Declare(Declare),
}

impl Display for BlockItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockItem::Statement(statement) => {
                write!(f, "{}", statement)
            }
            BlockItem::Declare(Declare(name, expression)) => {
                write!(f, "DECLARE {} = ", name)?;
                match expression {
                    Some(expr) => {
                        writeln!(f, "{}", expr)
                    }
                    None => {
                        writeln!(f, "")
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Declare(pub Expression, pub Option<Expression>);

#[derive(Debug)]
pub enum Statement {
    Return(Expression),
    If {
        pred: Expression,
        then: Box<Statement>,
        otherwise: Option<Box<Statement>>,
    },
    Expr(Expression),
}
impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Return(expression) => writeln!(f, "RETURN Int <{}>", expression),
            Statement::Expr(expression) => todo!(),
            Statement::If {
                pred,
                then,
                otherwise,
            } => todo!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Expression {
    Ident(String),
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
    Assign {
        ident: Box<Expression>,
        expr: Box<Expression>,
    },
    Conditional {
        pred: Box<Expression>,
        then: Box<Expression>,
        otherwise: Box<Expression>,
    },
}
#[derive(Debug, PartialEq, Eq, Hash)]
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

#[derive(Debug, PartialEq, Eq, Hash)]
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
    Modulo,
    BitwiseShiftLeft,
    BitwiseShiftRight,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
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
            BinaryOp::Modulo => write!(f, "%"),
            BinaryOp::BitwiseShiftLeft => write!(f, "<<"),
            BinaryOp::BitwiseShiftRight => write!(f, ">>"),
            BinaryOp::BitwiseAnd => write!(f, "&"),
            BinaryOp::BitwiseOr => write!(f, "|"),
            BinaryOp::BitwiseXor => write!(f, "^"),
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
            Expression::Ident(_) => todo!(),
            Expression::Assign { ident, expr } => todo!(),
            Expression::Conditional {
                pred,
                then,
                otherwise,
            } => todo!(),
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_tree(f, 0)
    }
}

pub trait MultipeekExt: Iterator {
    fn peek_n(&mut self, n: usize) -> Option<Self::Item>
    where
        Self::Item: Clone;
}

impl<T: Iterator> MultipeekExt for MultiPeek<T> {
    fn peek_n(&mut self, n: usize) -> Option<Self::Item>
    where
        Self::Item: Clone,
    {
        self.reset_peek();

        for _ in 0..(n - 1) {
            self.peek()?;
        }
        let item = self.peek().cloned();

        self.reset_peek();

        item
    }
}
