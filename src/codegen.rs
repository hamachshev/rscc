#![allow(unused)]

use crate::parser::{Expression, Function, Program, Statement};

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
    }
}
    }
}
