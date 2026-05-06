#![allow(unused)]

use crate::parser::{Expression, Function, Program, Statement};

pub fn gen_program(code: Program) -> String {
    gen_function(code.0)
}

fn gen_function(Function(name, statements): Function) -> String {
    let body: String = statements.into_iter().map(gen_statement).collect();
    format!("\t.globl _{name}\n_{name}:\n{body}")
}

fn gen_statement(statement: Statement) -> String {
    match statement {
        Statement::Return(expr) => {
            format!("\tmovl \t${}, %eax\n\tret\n", gen_expression(&expr))
        }
    }
}

fn gen_expression(expr: &Expression) -> String {
    match expr {
        Expression::Const(i) => i.to_string(),
    }
}
