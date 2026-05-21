#![allow(unused)]

use std::{collections::HashMap, fmt::format, path::Path};

use crate::parser::types::{
    BinaryOp, Block, BlockItem, Declare, Expression, Function, Program, Statement, UnaryOp,
};

pub struct CodeGen {
    label_counter: u32,
    var_map_stack: Vec<HashMap<String, u32>>,
    loop_ctx_stack: Vec<LoopCtx>,
    ebp_offset: u32, //in bytes - we mult by 8 later
}

impl CodeGen {
    pub fn new() -> CodeGen {
        CodeGen {
            label_counter: 0,
            var_map_stack: Vec::new(),
            loop_ctx_stack: Vec::new(),
            ebp_offset: 1,
        }
    }
    pub fn gen_program(&mut self, code: Program) -> String {
        self.gen_function(code.0)
    }

    fn gen_function(&mut self, Function(name, block): Function) -> String {
        let mut body = self.gen_block(&block);
        if block.0.len() != 0
            && let Some(BlockItem::Statement(Statement::Return(_))) = block.0.get(block.0.len() - 1)
        {
            //ie there is a return, then do nothing
        } else {
            body.push_str(&format!(
                "\tmovl \t$0, %eax\n \
                \tmov \t%rbp, %rsp\n \
                \tpop \t%rbp\n \
                \tret\n"
            ));
        };
        format!(
            "\t.globl _{name}\n_{name}:\n\
            \tpush \t%rbp\n\
            \tmov  \t%rsp, %rbp\n\
            {body}\n"
        )
    }
    fn gen_block(&mut self, Block(block): &Block) -> String {
        self.var_map_stack.push(HashMap::new());
        let mut body: String = block.iter().map(|x| self.gen_block_item(x)).collect();
        let var_map = self.var_map_stack.pop();
        let var_offset = var_map.unwrap().keys().len() as u32;
        self.ebp_offset -= var_offset;
        body.push_str(&format!("\taddq \t${}, %rsp\n", var_offset * 8));
        body
    }
    fn gen_block_item(&mut self, block_item: &BlockItem) -> String {
        match block_item {
            BlockItem::Statement(statement) => self.gen_statement(statement),
            BlockItem::Declare(declare) => self.gen_decl(declare),
            _ => panic!("unreadchable"),
        }
    }

    fn gen_decl(&mut self, declare: &Declare) -> String {
        let Declare(Expression::Ident(ident), expr) = declare else {
            panic!("non ident in declare - should not happen")
        };
        if self.var_map_stack.last().unwrap().contains_key(ident) {
            //get latest var
            //map
            panic!("cannot redeclare {ident}");
        }

        self.var_map_stack
            .last_mut()
            .unwrap()
            .insert(ident.clone(), self.ebp_offset);

        println!("{:?}....", self.var_map_stack);

        self.ebp_offset += 1;
        let expr = if let Some(expr) = expr {
            self.gen_expression(&expr)
        } else {
            "".to_string()
        };
        format!(
            "{expr}\
            \tpush \t%rax\n"
        )
    }

    fn gen_statement(&mut self, statement: &Statement) -> String {
        match statement {
            Statement::Return(expr) => {
                format!(
                    "{}\tmov \t%rbp, %rsp\n\
                    \tpop \t%rbp\n\
                    \tret\n",
                    self.gen_expression(&expr)
                )
            }
            Statement::Expr(expression) => self.gen_expression(&expression),
            Statement::If {
                pred,
                then,
                otherwise,
            } => {
                let pred = self.gen_expression(pred);
                let then = self.gen_statement(then);
                let else_label = format!("_label{}", self.label_counter);
                self.label_counter += 1;
                let otherwise = if let Some(otherwise) = otherwise {
                    self.gen_statement(otherwise)
                } else {
                    "".to_string()
                };
                let end_label = format!("_label{}", self.label_counter);
                self.label_counter += 1;

                let mut output = format!(
                    "{pred}\
                    \tcmpl \t$0, %eax\n\
                    \tje \t{else_label}\n\
                    {then}\
                    \tjmp \t{end_label}\n\
                    {else_label}:\n\
                    {otherwise}\
                    {end_label}:\n\
                    "
                );
                output
            }
            Statement::Block(block) => self.gen_block(&block),
            Statement::For {
                init,
                condition,
                post,
                body,
            } => {
                let condition_label = self.make_label();
                let post_label = self.make_label();
                let end_label = self.make_label();

                let ctx = LoopCtx {
                    continue_label: post_label.clone(),
                    break_label: end_label.clone(),
                };
                self.loop_ctx_stack.push(ctx);

                let init = init
                    .as_ref()
                    .map(|i| self.gen_expression(i))
                    .unwrap_or_default();
                let condition = self.gen_expression(condition);
                let post = post
                    .as_ref()
                    .map(|ref i| self.gen_expression(i))
                    .unwrap_or_default();
                let body = self.gen_statement(body);

                self.loop_ctx_stack.pop();

                format!(
                    "{init}\
                {condition_label}:\n\
                {condition}\
                \tje \t{end_label}\n\
                {body}\
                {post_label}:\n\
                {post}\
                \tjmp \t{condition_label}\n\
                {end_label}:\n\
                "
                )
            }
            Statement::ForDecl {
                init,
                condition,
                post,
                body,
            } => {
                let condition_label = self.make_label();
                let post_label = self.make_label();
                let end_label = self.make_label();

                let ctx = LoopCtx {
                    continue_label: post_label.clone(),
                    break_label: end_label.clone(),
                };
                self.loop_ctx_stack.push(ctx);
                self.var_map_stack.push(HashMap::new());

                let init = init.as_ref().map(|i| self.gen_decl(i)).unwrap_or_default();
                let condition = self.gen_expression(condition);
                let post = post
                    .as_ref()
                    .map(|ref i| self.gen_expression(i))
                    .unwrap_or_default();
                let body = self.gen_statement(body);

                self.loop_ctx_stack.pop();
                self.var_map_stack.pop();
                self.ebp_offset -= 1; // this is assuming only one decl in the for loop

                format!(
                    "{init}\
                {condition_label}:\n\
                {condition}\
                \tje \t{end_label}\n\
                {body}\
                {post_label}:\n\
                {post}\
                \tjmp \t{condition_label}\n\
                {end_label}:\n\
                \taddq \t$8, %rsp\n\
                "
                ) // this is assuming only one decl in the for loop
            }
            Statement::While { condition, body } => {
                let condition_label = self.make_label();
                let end_label = self.make_label();

                let ctx = LoopCtx {
                    continue_label: condition_label.clone(),
                    break_label: end_label.clone(),
                };
                self.loop_ctx_stack.push(ctx);

                let condition = self.gen_expression(condition);
                let body = self.gen_statement(body);

                self.loop_ctx_stack.pop();

                format!(
                    "{condition_label}:\n\
                    {condition}\
                    \tcmpl \t$0, %eax\n\
                    \tje \t{end_label}\n\
                    {body}\
                    \tjmp \t{condition_label}\n\
                    {end_label}:\n\
                    "
                )
            }
            Statement::Do { body, condition } => {
                let body_label = self.make_label();
                let end_label = self.make_label();

                let ctx = LoopCtx {
                    continue_label: body_label.clone(),
                    break_label: end_label.clone(),
                };
                self.loop_ctx_stack.push(ctx);

                let body = self.gen_statement(body);
                let condition = self.gen_expression(condition);

                self.loop_ctx_stack.pop();

                format!(
                    "{body_label}:\n\
                    {body}\
                    {condition}\
                    \tcmpl \t$0, %eax\n\
                    \tje \t{end_label}\n\
                    \tjmp \t{body_label}\n\
                    {end_label}:\n\
                    "
                )
            }
            Statement::Break => {
                let LoopCtx { break_label, .. } = self
                    .loop_ctx_stack
                    .last()
                    .expect("cant use break outside of a loop");
                format!("\tjmp \t{break_label}\n")
            }
            Statement::Continue => {
                let LoopCtx { continue_label, .. } = self
                    .loop_ctx_stack
                    .last()
                    .expect("cant use break outside of a loop");
                format!("\tjmp \t{continue_label}\n")
            }
            Statement::Null => "".to_string(),
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
                            BinaryOp::LT => "\tsetl \t%al\n",
                            BinaryOp::LTE => "\tsetle \t%al\n",
                            BinaryOp::GT => "\tsetg \t%al\n",
                            BinaryOp::GTE => "\tsetge \t%al\n",
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
                println!(
                    "using var {:?}: from map {:?}",
                    ident,
                    self.var_map_stack
                        .iter()
                        .rev()
                        .find(|m| m.contains_key(ident))
                        .unwrap()
                );
                let offset = *(self
                    .var_map_stack
                    .iter()
                    .rev()
                    .find_map(|m| m.get(ident))
                    .expect(&format!("using {ident} before you declared it")))
                    as i32
                    * -8;
                format!("\tmovl \t{offset}(%rbp), %eax\n")
            }
            Expression::Assign { ident, expr } => {
                let Expression::Ident(ident) = ident.as_ref() else {
                    panic!("non ident expression for assign expression")
                };
                let offset = *(self
                    .var_map_stack
                    .iter()
                    .rev()
                    .find_map(|m| m.get(ident))
                    .expect(&format!("must declare {} before assignment", ident)))
                    as i32
                    * -8;

                let expr = self.gen_expression(expr);
                format!("{expr}\tmovl \t%eax, {offset}(%rbp)\n")
            }
            Expression::Conditional {
                pred,
                then,
                otherwise,
            } => {
                let pred = self.gen_expression(pred);
                let then = self.gen_expression(then);
                let otherwise = self.gen_expression(otherwise);
                let otherwise_label = format!("_label{}", self.label_counter);
                self.label_counter += 1;
                let end_label = format!("_label{}", self.label_counter);
                self.label_counter += 1;

                format!(
                    "{pred}\
                    \tcmpl \t$0, %eax\n\
                    \tje \t{otherwise_label}\n\
                    {then}\
                    \tjmp \t{end_label}\n\
                    {otherwise_label}:\n\
                    {otherwise}\
                    {end_label}:\n"
                )
            }
            Expression::Null => "".to_string(),
        }
    }
    fn make_label(&mut self) -> String {
        let label = format!("_label{}", self.label_counter);
        self.label_counter += 1;
        label
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

struct LoopCtx {
    continue_label: String,
    break_label: String,
}
