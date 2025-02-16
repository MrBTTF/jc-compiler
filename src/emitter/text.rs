pub mod abi;
pub mod mnemonics;

use std::{
    collections::{BTreeMap, HashMap},
    mem,
};

pub use code_context::*;

use super::{
    ast::{self, *},
    data::{Data, DataBuilder, DataType},
    stack::StackManager,
};
use mnemonics::*;

mod code_context;
mod stdlib;

pub fn build_code_context(
    statement_list: &ast::StatementList,
    symbol_data: &HashMap<String, Data>,
    scope_symbols: &HashMap<String, Vec<String>>,
    image_base: u64,
) -> CodeContext {
    let mut text_builder = TextBuilder::new(symbol_data, scope_symbols, image_base);
    text_builder.visit_ast(statement_list);
    text_builder.get_code_context()
}

pub struct TextBuilder {
    code_context: CodeContext,
    symbol_data: HashMap<String, Data>,
    scope_symbols: HashMap<String, Vec<String>>,
    stack_manager: StackManager,
}

impl TextBuilder {
    pub fn new(
        symbol_data: &HashMap<String, Data>,
        scope_symbols: &HashMap<String, Vec<String>>,
        image_base: u64,
    ) -> Self {
        TextBuilder {
            code_context: CodeContext::new(image_base),
            symbol_data: symbol_data.clone(),
            scope_symbols: scope_symbols.clone(),
            stack_manager: StackManager::new(),
        }
    }

    pub fn get_code_context(&self) -> CodeContext {
        self.code_context.clone()
    }

    fn visit_ast(&mut self, statement_list: &ast::StatementList) {
        self.code_context.add_slice(&self.stack_manager.new_stack());

        let call = self.call("main");
        self.code_context.add_slice(&call);
        stdlib::exit(&mut self.code_context, 0);

        self.visit_statement_list(statement_list);
    }

    fn visit_statement_list(&mut self, statement_list: &ast::StatementList) {
        let mnemonics = self.allocate_stack(&statement_list);
        self.code_context.add_slice(&mnemonics);
        statement_list.stmts.iter().for_each(|stmt| {
            self.visit_statement(stmt, &statement_list.id);
        });

        // self.code_context.add_slice(&self.stack_manager.free());
    }

    fn visit_statement(&mut self, statement: &ast::Statement, scope: &str) {
        // println!("{}: {:#?}", statement_n, &statement);
        match statement {
            ast::Statement::FuncDefinition(func_def) => {
                self.visit_func_definition(func_def);
            }
            ast::Statement::Expression(expr) => {
                self.visit_expression(expr, scope);
            }
            ast::Statement::Declaration(_) => (),
            ast::Statement::Assignment(assign) => {
                self.visit_assignment(assign, scope);
            }
            Statement::Scope(stmts) => (),
            Statement::ControlFlow(_) => (),
        };
    }

    fn visit_func_definition(&mut self, func_def: &ast::FuncDefinition) {
        let ast::FuncDefinition(name, args, return_type, stmt_list) = func_def;

        self.code_context.set_label(name.value.clone());
        self.code_context
            .add_slice(&self.stack_manager.init_function_stack());

        args.iter()
            .rev()
            .enumerate()
            .for_each(|(i, arg)| match arg._type {
                ast::Type::Ref(_) => (),
                _ => {
                    self.code_context
                        .add_slice(&self.stack_manager.push_register(abi::ARG_REGISTERS[i]));
                }
            });

        self.visit_statement_list(stmt_list);

        args.iter()
            .enumerate()
            .for_each(|(i, arg)| match arg._type {
                ast::Type::Ref(_) => (),
                _ => {
                    self.code_context
                        .add_slice(&self.stack_manager.pop_register(abi::ARG_REGISTERS[i]));
                }
            });

        self.code_context
            .add_slice(&self.stack_manager.drop_function_stack());

        self.code_context.add(RET.no_op());
    }

    fn visit_expression(&mut self, expr: &ast::Expression, scope: &str) {
        match expr {
            ast::Expression::Call(id, exprs) => {
                self.visit_call(id, exprs, scope);
            }
            ast::Expression::Loop(l) => self.visit_loop(l, scope),
            _ => (),
        }
    }

    fn visit_assignment(&mut self, assign: &ast::Assignment, scope: &str) {
        let ast::Assignment(id, expr) = assign;

        match expr {
            Expression::Literal(lit) => {
                let data = self
                    .get_symbol_data(&scope, &id.value)
                    .unwrap_or_else(|| panic!("undefined variable: {}", id.value))
                    .clone();
                if data.decl_type == DeclarationType::Const {
                    panic!("Cannot assign to const data: {data:#?}");
                }
                let lit_data_type: DataType = lit.clone().into();
                if std::mem::discriminant(&data.data_type) != std::mem::discriminant(&lit_data_type)
                {
                    panic!("Cannot assign {:?} to {:?}", data.data_type, lit_data_type);
                }

                // data.set_data_type(lit.clone().into());

                self.code_context.add_slice(&match lit {
                    Literal::String(s) => self.stack_manager.push_list(&str_to_u64(s), s.len()),
                    Literal::Number(n) => self.stack_manager.push(n.value as u64),
                });

                let data_loc = self.stack_manager.get_local_top();

                let mut data_size = data.data_size;
                let remainder = data_size % 8;
                if remainder != 0 {
                    data_size += 8 - remainder;
                }

                let data = self
                    .get_symbol_data_mut(&scope, &id.value)
                    .unwrap_or_else(|| panic!("undefined variable: {}", id.value));
                data.data_size = data_size;
                data.data_loc = data_loc as u64;

                dbg!(&lit);
                dbg!(&data);
            }
            Expression::Ident(_) => todo!(),
            Expression::Call(_, _) => todo!(),
            Expression::Loop(_) => todo!(),
        };
    }

    fn visit_call(&mut self, id: &ast::Ident, exprs: &[ast::Expression], scope: &str) {
        if id.value == "print" {
            let data = match exprs.first() {
                Some(ast::Expression::Ident(id)) => self
                    .get_symbol_data(&scope, &id.value)
                    .unwrap_or_else(|| panic!("undefined variable: {}::{}", &scope, id.value))
                    .clone(),
                _ => panic!("Function print expects on argument"),
            };
            dbg!(&data);
            match data.data_type {
                DataType::String(_) => {
                    let args = vec![data.clone()];

                    abi::push_args(
                        &mut self.code_context,
                        &mut self.stack_manager,
                        args.as_slice(),
                    );

                    stdlib::print(&mut self.code_context, data);

                    abi::pop_args(&mut self.code_context, &mut self.stack_manager, args.len());
                }
                DataType::Int(n) => {
                    let format = self
                        .symbol_data
                        .get("global::__printf_d_arg")
                        .unwrap_or_else(|| panic!("undefined variable: global::__printf_d_arg"));

                    let args = &[format.clone(), data.clone()];

                    abi::push_args(&mut self.code_context, &mut self.stack_manager, args);

                    self.code_context
                        .add_slice(&self.stack_manager.align_for_call());

                    stdlib::printd(&mut self.code_context);

                    self.code_context
                        .add_slice(&self.stack_manager.unalign_after_call());

                    abi::pop_args(&mut self.code_context, &mut self.stack_manager, args.len());
                }
            };

            return;
        } else {
            let args: Vec<Data> = exprs
                .iter()
                .flat_map(|expr| {
                    let id: &Ident = match expr {
                        Expression::Ident(id) => id,
                        _ => todo!(),
                    };
                    let data = self
                        .get_symbol_data(&scope, &id.value)
                        .unwrap_or_else(|| panic!("undefined symbol: {}", id.value))
                        .clone();

                    vec![data]
                })
                .collect();

            abi::push_args(
                &mut self.code_context,
                &mut self.stack_manager,
                args.as_slice(),
            );

            let call_code = self.call(&id.value);
            self.code_context.add_slice(&call_code);
            abi::pop_args(&mut self.code_context, &mut self.stack_manager, args.len());
        }
    }

    fn visit_loop(&mut self, l: &ast::Loop, scope: &str) {
        let counter = self
            .get_symbol_data(&l.body.id, &l.var.value)
            .unwrap_or_else(|| panic!("undefined variable: {}::{}", l.body.id, l.var.value))
            .clone();

        self.code_context.add_slice(&self.stack_manager.new_stack());

        let statement_list = &l.body;

        let mnemonics = self.allocate_stack(&statement_list);
        self.code_context.add_slice(&mnemonics);

        let offset = self.code_context.get_code_size();
        statement_list.stmts.iter().for_each(|stmt| {
            self.visit_statement(stmt, &statement_list.id);
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

        self.code_context.add_slice(&self.stack_manager.drop());
    }

    fn allocate_stack(&mut self, stmts: &ast::StatementList) -> Vec<Mnemonic> {
        let ids = self.scope_symbols.get(&stmts.id).unwrap().to_vec();

        let mut code = vec![];
        for id in ids.iter() {
            let data = self.symbol_data.get(id).unwrap();

            code.extend(match data.decl_type {
                ast::DeclarationType::Let => match &data.data_type {
                    DataType::String(s) => self.stack_manager.push_list(&str_to_u64(s), s.len()),
                    DataType::Int(i) => self.stack_manager.push(*i as u64),
                },
                ast::DeclarationType::Const => vec![],
            });
        }
        // dbg!(self.stack_manager.get_top());
        // dbg!(&code);
        code
    }

    fn get_symbol_data(&self, scope: &str, id: &str) -> Option<&Data> {
        let id = format!("{}::{}", scope, id);
        // dbg!(&scope, &id);

        self.symbol_data.get(&id)
    }

    fn get_symbol_data_mut(&mut self, scope: &str, id: &str) -> Option<&mut Data> {
        let id = format!("{}::{}", scope, id);
        // dbg!(&scope, &id);

        self.symbol_data.get_mut(&id)
    }

    fn call(&mut self, label: &str) -> Vec<Mnemonic> {
        let mut code = self.stack_manager.align_for_call();
        code.push(CALL.op1(Operand::Offset32(0)).symbol(label.to_string()));
        code.extend(self.stack_manager.unalign_after_call());

        code
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
