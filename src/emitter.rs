use ::std::collections::BTreeMap;
use ::std::path::PathBuf;

use ast::*;
use code_context::CodeContext;
use data::{Data, DataBuilder};
#[cfg(target_os = "linux")]
use elf::build;
#[cfg(target_os = "linux")]
use elf::sections::VIRTUAL_ADDRESS_START as IMAGE_BASE;

#[cfg(target_os = "windows")]
use exe::build;
#[cfg(target_os = "windows")]
use exe::sections::IMAGE_BASE;

use mnemonics::*;

mod abi;
#[cfg(target_os = "linux")]
use abi::linux as call_abi;
#[cfg(target_os = "windows")]
use abi::windows as call_abi;

pub mod ast;
pub mod code_context;
mod data;
#[cfg(target_os = "linux")]
pub mod elf;
#[cfg(target_os = "windows")]
pub mod exe;
mod mnemonics;

mod stdlib;
#[cfg(target_os = "linux")]
use stdlib::linux as std;
#[cfg(target_os = "windows")]
use stdlib::windows as std;

pub fn build_executable(ast: &ast::StatementList, output_path: PathBuf) {
    let mut data_builder = DataBuilder::default();
    data_builder.visit_ast(ast);
    dbg!(&data_builder.variables);

    let mut emitter = Emitter::new(&data_builder, IMAGE_BASE);
    emitter.visit_ast(ast);

    build(output_path, emitter, &data_builder.variables);
}

pub struct Emitter {
    code_context: CodeContext,
    literals: BTreeMap<ast::Ident, Data>,
    data_ordered: Vec<ast::Ident>,
}

impl Emitter {
    fn new(data_builder: &DataBuilder, image_base: u64) -> Self {
        Emitter {
            code_context: CodeContext::new(image_base),
            literals: data_builder.variables.clone(),
            data_ordered: data_builder.data_ordered.clone(),
        }
    }

    pub fn get_code_context(&self) -> CodeContext {
        self.code_context.clone()
    }

    fn visit_ast(&mut self, statement_list: &ast::StatementList) {
        self.code_context.add_slice(&[
            PUSH.op1(register::RBP),
            MOV.op1(register::RBP).op2(register::RSP),
        ]);
        self.code_context.add(
            CALL.op1(Operand::Offset32(0))
                .symbol("main".to_string(), CallType::Local),
        );
        std::exit(&mut self.code_context, 0);

        self.visit_statement_list(statement_list);
    }

    fn visit_statement_list(&mut self, statement_list: &ast::StatementList) {
        statement_list.0.iter().for_each(|stmt| {
            self.visit_statement(stmt);
        });
    }

    fn visit_statement(&mut self, statement: &ast::Statement) {
        // println!("{}: {:#?}", statement_n, &statement);
        match statement {
            ast::Statement::FuncDefinition(func_def) => {
                self.visit_func_definition(func_def);
            }
            ast::Statement::Expression(expr) => {
                self.visit_expression(expr);
            }
            ast::Statement::Declaration(_) => (),
            ast::Statement::Assignment(assign) => {
                self.visit_assignment(assign);
            }
            Statement::ControlFlow(_) => (),
        };
    }

    fn visit_func_definition(&mut self, func_def: &ast::FuncDefinition) {
        let ast::FuncDefinition(name, args, return_type, stmt_list) = func_def;

        self.code_context.set_label(name.value.clone());
        self.code_context.add_slice(&[
            PUSH.op1(register::RBP),
            MOV.op1(register::RBP).op2(register::RSP),
        ]);

        let (mnemonics, size) = self.allocate_stack();
        self.code_context.add_slice(&mnemonics);
        self.visit_statement_list(stmt_list);
        self.code_context.add_slice(&self.free_stack(size));
        self.code_context.add(POP.op1(register::RBP));
        self.code_context.add(RET.no_op());
    }

    fn visit_expression(&mut self, expr: &ast::Expression) {
        match expr {
            ast::Expression::Call(id, expr) => {
                self.visit_call(id, expr);
            }
            ast::Expression::Loop(l) => self.visit_loop(l),
            _ => (),
        }
    }

    fn visit_assignment(&mut self, assign: &ast::Assignment) {
        let ast::Assignment(id, expr) = assign;

        match expr {
            Expression::Literal(lit) => {
                let values = match lit {
                    Literal::String(s) => str_to_u64(s),
                    Literal::Number(n) => vec![n.value as u64],
                };

                let data = self
                    .literals
                    .get_mut(id)
                    .unwrap_or_else(|| panic!("undefined variable: {}", id.value));
                if data.decl_type == DeclarationType::Const {
                    panic!("Cannot assign to const data: {data:#?}");
                }
                data.lit = lit.clone();

                let assign_at_stack_location =
                    values
                        .iter()
                        .enumerate()
                        .fold(vec![], |mut acc: Vec<Mnemonic>, (i, v)| {
                            acc.push(MOV.op1(register::RAX).op2(*v));
                            acc.push(PUSH.op1(register::RAX));
                            acc
                        });

                self.code_context.add_slice(&[
                    PUSH.op1(register::RAX),
                    PUSH.op1(register::RBX),
                    MOV.op1(register::RBX).op2(register::RSP),
                    MOV.op1(register::RSP).op2(register::RBP),
                ]);
                self.code_context
                    .add_slice(assign_at_stack_location.as_slice());
                self.code_context.add_slice(&[
                    MOV.op1(register::RSP).op2(register::RBX),
                    POP.op1(register::RBX),
                    POP.op1(register::RAX),
                ]);
            }
            Expression::Ident(_) => todo!(),
            Expression::Call(_, _) => todo!(),
            Expression::Loop(_) => todo!(),
        };
    }

    fn visit_call(&mut self, id: &ast::Ident, expr: &ast::Expression) {
        let data = match expr {
            ast::Expression::Ident(id) => self
                .literals
                .get(id)
                .unwrap_or_else(|| panic!("undefined variable: {}", id.value)),
            _ => todo!(),
        };

        if id.value == "print" {
            match data.lit {
                ast::Literal::String(_) => {
                    let args = &[data.clone()];
                    call_abi::push_args(&mut self.code_context, args);

                    std::print(&mut self.code_context, data.clone());

                    call_abi::pop_args(&mut self.code_context, args.len());
                }
                ast::Literal::Number(n) => {
                    let format = self
                        .literals
                        .get(&ast::Ident {
                            value: "__printf_d_arg".to_string(),
                        })
                        .unwrap_or_else(|| panic!("undefined variable: {}", id.value));

                    let args = &[format.clone(), data.clone()];

                    call_abi::push_args(&mut self.code_context, args);

                    std::printd(&mut self.code_context);

                    call_abi::pop_args(&mut self.code_context, args.len());
                }
            };

            return;
        }

        panic!("no such function {}", id.value)
    }

    fn visit_loop(&mut self, l: &ast::Loop) {
        let counter = self
            .literals
            .get(&l.var)
            .unwrap_or_else(|| panic!("undefined variable: {}", l.var.value))
            .clone();

        let offset = self.code_context.get_code_size();

        l.body.iter().for_each(|stmt| {
            self.visit_statement(stmt);
        });
        self.code_context
            .add(MOV.op1(register::RCX).op2(register::RBP));
        self.code_context
            .add(SUB.op1(register::RCX).op2(counter.data_loc as u32));
        self.code_context
            .add(INC.op1(register::RCX).disp(Operand::Offset32(0)));
        self.code_context.add(
            CMP.op1(register::RCX)
                .op2(l.end as u32)
                .disp(Operand::Offset32(0)),
        );

        let jump = JL.op1(Operand::Offset32(-(0 as i32))).as_vec().len()
            + self.code_context.get_code_size()
            - offset;
        dbg!(jump);
        self.code_context
            .add(JL.op1(Operand::Offset32(-(jump as i32))));
    }

    fn allocate_stack(&self) -> (Vec<Mnemonic>, usize) {
        let mut result = vec![];
        let mut size = 0;
        for id in self.data_ordered.iter() {
            let data = self.literals.get(id).unwrap();
            if data.decl_type == ast::DeclarationType::Const {
                continue;
            }
            result.extend(match data.decl_type {
                ast::DeclarationType::Let => match &data.lit {
                    ast::Literal::String(s) => {
                        let (mnemonics, pushed_size) = push_string_on_stack(&s);
                        size += pushed_size;
                        mnemonics
                    }
                    ast::Literal::Number(n) => {
                        size += 8;

                        vec![
                            MOV.op1(register::RAX).op2(n.value as u64),
                            PUSH.op1(register::RAX),
                        ]
                    }
                },
                ast::DeclarationType::Const => vec![],
            });
        }

        if result.len() % 4 != 0 {
            size += 8;
            result.push(SUB.op1(register::RSP).op2(8_u32));
        }
        (result, size)
    }

    fn free_stack(&self, size: usize) -> Vec<Mnemonic> {
        vec![ADD.op1(register::RSP).op2(size as u32)]
    }
}

fn str_to_u64(s: &str) -> Vec<u64> {
    s.as_bytes()
        .chunks(8)
        .rev()
        .fold(vec![], |mut acc: Vec<u64>, substr| {
            let mut value: u64 = 0;
            for (i, c) in substr.iter().enumerate() {
                value += (*c as u64) << (8 * i)
            }
            acc.push(value);
            acc
        })
}

fn push_string_on_stack(s: &str) -> (Vec<Mnemonic>, usize) {
    let mut size = 0;
    let mnemonics = str_to_u64(s)
        .iter()
        .fold(vec![], |mut acc: Vec<Mnemonic>, value| {
            acc.push(MOV.op1(register::RAX).op2(*value));
            acc.push(PUSH.op1(register::RAX));
            size += 8;
            acc
        });

    (mnemonics, size)
}
