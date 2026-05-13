use std::fmt::Display;

#[derive(Debug)]
pub struct Program(pub Function);

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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
pub enum Statement {
    Return(Expression),
    Declare(Expression, Option<Expression>),
    Expr(Expression),
}
impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Return(expression) => writeln!(f, "RETURN Int <{}>", expression),
            Statement::Declare(name, expression) => {
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
            Statement::Expr(expression) => todo!(),
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
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_tree(f, 0)
    }
}
