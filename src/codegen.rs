#![allow(unused)]

use std::fmt::format;

use crate::parser::{self, BinaryOp, Expression, Function, Program, Statement};

struct CodeGen {
    label_counter: u32,
}

pub fn gen_program(code: Program) -> String {
    CodeGen::new().gen_function(code.0)
}

impl CodeGen {
    fn new() -> CodeGen {
        CodeGen { label_counter: 0 }
    }

    fn gen_function(&mut self, Function(name, statements): Function) -> String {
        let body: String = statements
            .0
            .into_iter()
            .map(|x| self.gen_statement(x))
            .collect();
        format!("\t.globl _{name}\n_{name}:\n{body}\n")
    }

    fn gen_statement(&mut self, statement: Statement) -> String {
        match statement {
            Statement::Return(expr) => {
                format!("{}\tret", self.gen_expression(&expr))
            }
        }
    }

    fn gen_expression(&mut self, expr: &Expression) -> String {
        match expr {
            Expression::Const(i) => format!("\tmovl \t${}, %eax\n", i),
            Expression::Unary { op, expr } => match op {
                parser::UnaryOp::Negation => {
                    let operand = self.gen_expression(expr);
                    format!("{operand}\tneg \t%eax\n")
                }
                parser::UnaryOp::LogicalNegation => {
                    let operand = self.gen_expression(expr);
                    format!("{operand}\tcmpl\t$0, %eax\n\tmovl\t$0, %eax\n\tsete\t%al\n")
                }
                parser::UnaryOp::BitwiseComplement => {
                    let operand = self.gen_expression(expr);
                    format!("{operand}\tnot \t%eax\n")
                }
            },
            Expression::Binary { op, l_expr, r_expr } => {
                let lhs = self.gen_expression(l_expr);
                let rhs = self.gen_expression(r_expr);
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
                    BinaryOp::And => {
                        let rhs_label = format!("_label{}", self.label_counter);
                        self.label_counter += 1;

                        let end_label = format!("_label{}", self.label_counter);
                        self.label_counter += 1;
                        format!(
                            "{lhs}\tcmpl \t$0, %eax\n\tjne \t{rhs_label}\n\tjmp \t{end_label}\n{rhs_label}:\n{rhs}\tcmpl $0, %eax\n\tmovl \t$0, %eax\n\tsetne \t%al\n{end_label}:\n"
                        )
                    }
                    BinaryOp::Or => {
                        let rhs_label = format!("_label{}", self.label_counter);
                        self.label_counter += 1;

                        let end_label = format!("_label{}", self.label_counter);
                        self.label_counter += 1;
                        format!(
                            "{lhs}\tcmpl \t$0, %eax\nje \t{rhs_label}\n\tmovl \t$1, %eax\n\tjmp \t{end_label}\n{rhs_label}:\n{rhs}\tcmpl \t$0, %eax\n\tmovl \t$0, %eax\n\tsetne %al\n{end_label}:\n"
                        )
                    }
                    BinaryOp::Equal => {
                        format!(
                            "{lhs}\tpush \t%rax\n{rhs}\tpop \t%rcx\n\tcmpl \t%ecx, %eax\n\tmovl \t$0, %eax\n\tsete %al\n"
                        )
                    }
                    BinaryOp::NotEqual => {
                        format!(
                            "{lhs}\tpush \t%rax\n{rhs}\tpop \t%rcx\n\tcmpl \t%ecx, %eax\n\tmovl \t$0, %eax\n\tsetne %al\n"
                        )
                    }
                    BinaryOp::LT | BinaryOp::LTE | BinaryOp::GT | BinaryOp::GTE => {
                        let mut asm = format!(
                            "{rhs}\tpush \t%rax\n{lhs}\tpop \t%rcx\n\tcmpl \t%ecx, %eax\n\tmovl \t$0, %eax\n"
                        );
                        asm.push_str(match op {
                            BinaryOp::LT => "\tsetl %al\n",
                            BinaryOp::LTE => "\tsetle %al\n",
                            BinaryOp::GT => "\tsetg %al\n",
                            BinaryOp::GTE => "\tsetge %al\n",
                            _ => panic!("should not happen"),
                        });
                        asm
                    }
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
