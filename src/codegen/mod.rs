#![allow(unused)]

use std::{collections::HashMap, fmt::format, path::Path};

use crate::parser::types::{BinaryOp, Expression, Function, Program, Statement, UnaryOp};

pub struct CodeGen {
    label_counter: u32,
    var_map: HashMap<String, u32>,
    ebp_offset: u32, //in bytes - we mult by 4 later
}

impl CodeGen {
    pub fn new() -> CodeGen {
        CodeGen {
            label_counter: 0,
            var_map: HashMap::new(),
            ebp_offset: 1,
        }
    }
    pub fn gen_program(&mut self, code: Program) -> String {
        self.gen_function(code.0)
    }

    fn gen_function(&mut self, Function(name, statements): Function) -> String {
        let mut body: String = statements.0.iter().map(|x| self.gen_statement(x)).collect();
        if let Some(Statement::Return(_)) = statements.0.get(statements.0.len() - 1) {
            //ie there is a return, then do nothing
        } else {
            body.push_str(&format!("movl \t$0, %eax\n\tret\n"));
        };
        format!(
            "\t.globl _{name}\n_{name}:\n\
            \tpush \t%rbp\n\
            \tmov  \t%rsp, %rbp\n\
            {body}\n"
        )
    }

    fn gen_statement(&mut self, statement: &Statement) -> String {
        match statement {
            Statement::Return(expr) => {
                format!(
                    "{}\
                    \tmov \t%rbp, %rsp\n\
                    \tpop \t%rbp\n\
                    \tret",
                    self.gen_expression(&expr)
                )
            }
            Statement::Declare(Expression::Ident(ident), expr) => {
                if self.var_map.contains_key(ident) {
                    panic!("cannot redeclare {ident}");
                }

                self.var_map.insert(ident.clone(), self.ebp_offset);
                self.ebp_offset += 1;
                if let Some(expr) = expr {
                    let expr = self.gen_expression(&expr);
                    format!("{expr}\tpush \t%rax\n")
                } else {
                    "".to_string()
                }
            }
            Statement::Expr(expression) => self.gen_expression(&expression),
            _ => panic!("unreadchable"),
        }
    }

    fn gen_expression(&mut self, expr: &Expression) -> String {
        match expr {
            Expression::Const(i) => format!("\tmovl \t${}, %eax\n", i),
            Expression::Unary { op, expr } => match op {
                UnaryOp::Negation => {
                    let operand = self.gen_expression(expr);
                    format!("{operand}\tneg \t%eax\n")
                }
                UnaryOp::LogicalNegation => {
                    let operand = self.gen_expression(expr);
                    format!("{operand}\tcmpl\t$0, %eax\n\tmovl\t$0, %eax\n\tsete\t%al\n")
                }
                UnaryOp::BitwiseComplement => {
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
                    BinaryOp::Modulo => {
                        format!(
                            "{rhs}\tpush \t%rax\n{lhs}\tcdq\n\tpop \t%rcx\n\tidiv \t%ecx\n\tmovl \t%edx, %eax\n"
                        )
                    }
                    BinaryOp::BitwiseShiftLeft => {
                        format!("{rhs}\tpush \t%rax\n{lhs}\tpop \t%rcx\n\tsal \t%cl, %eax\n")
                    }
                    BinaryOp::BitwiseShiftRight => {
                        format!("{rhs}\tpush \t%rax\n{lhs}\tpop \t%rcx\n\tshr \t%cl, %eax\n")
                    }
                    BinaryOp::BitwiseAnd => {
                        format!("{rhs}\tpush \t%rax\n{lhs}\tpop \t%rcx\n\tand \t%ecx, %eax\n")
                    }
                    BinaryOp::BitwiseOr => {
                        format!("{rhs}\tpush \t%rax\n{lhs}\tpop \t%rcx\n\tor \t%ecx, %eax\n")
                    }
                    BinaryOp::BitwiseXor => {
                        format!("{rhs}\tpush \t%rax\n{lhs}\tpop \t%rcx\n\txor \t%ecx, %eax\n")
                    }
                }
            }
            Expression::Ident(ident) => {
                let offset = *(self
                    .var_map
                    .get(ident)
                    .expect(&format!("using {ident} before you declared it")))
                    as i32
                    * -4;
                format!("movl \t{offset}(%ebp), %eax\n")
            }
            Expression::Assign { ident, expr } => {
                let Expression::Ident(ident) = ident.as_ref() else {
                    panic!("non ident expression for assign expression")
                };
                let offset = *(self
                    .var_map
                    .get(ident)
                    .expect(&format!("must declare {} before assignment", ident)))
                    as i32
                    * -4;

                let expr = self.gen_expression(expr);
                format!("{expr}\n movl \t%eax, {offset}(%ebp)\n")
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
