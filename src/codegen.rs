#![allow(unused)]

use crate::parser::{BinaryOp, Expression, Function, Program, Statement};

pub fn gen_program(code: Program) -> String {
    gen_function(code.0)
}

fn gen_function(Function(name, statements): Function) -> String {
    let body: String = statements.0.into_iter().map(gen_statement).collect();
    format!("\t.globl _{name}\n_{name}:\n{body}\n")
}

fn gen_statement(statement: Statement) -> String {
    match statement {
        Statement::Return(expr) => {
            format!("{}\tret", gen_expression(&expr))
        }
    }
}

fn gen_expression(expr: &Expression) -> String {
    match expr {
        Expression::Const(i) => format!("\tmovl \t${}, %eax\n", i),
        Expression::Unary { op, expr } => match op {
            crate::parser::UnaryOp::Negation => {
                let operand = gen_expression(expr);
                format!("{operand}\tneg \t%eax\n")
            }
            crate::parser::UnaryOp::LogicalNegation => {
                let operand = gen_expression(expr);
                format!("{operand}\tcmpl\t$0, %eax\n\tmovl\t$0, %eax\n\tsete\t%al\n")
            }
            crate::parser::UnaryOp::BitwiseComplement => {
                let operand = gen_expression(expr);
                format!("{operand}\tnot \t%eax\n")
            }
        },
        Expression::Binary { op, l_expr, r_expr } => {
            let lhs = gen_expression(l_expr);
            let rhs = gen_expression(r_expr);
            match op {
                BinaryOp::Add => {
                    format!("{lhs}\tpush \t%rax\n{rhs}\tpop \t%rcx\n\taddl \t%ecx, %eax\n")
                }
                BinaryOp::Mul => {
                    format!("{lhs}\tpush \t%rax\n{rhs}\tpop \t%rcx\n\timul \t%ecx, %eax\n")
                }
                BinaryOp::Div => {
                    format!("{rhs}\tpush \t%rax\n{lhs}\n\tcdq\n\tpop \t%rcx\n\tidiv \t%ecx\n")
                }
                BinaryOp::Sub => {
                    format!("{rhs}\tpush \t%rax\n{lhs}\tpop \t%rcx\n\tsubl \t%ecx, %eax\n") // subl
                    // source, dest = dest - source
                }
            }
        }
    }
}

trait AsmBuilder {
    fn movl(self, value: &str, reg: &str) -> Self;
    fn negate(self, reg: &str) -> Self;
}

impl AsmBuilder for String {
    fn movl(mut self, value: &str, reg: &str) -> Self {
        self.push_str(&format!("\tmovl\t${value}, %{reg}\n"));
        self
    }

    fn negate(mut self, reg: &str) -> Self {
        self.push_str(&format!("\tneg\t{reg}\n"));
        self
    }
}
