pub mod abi;
pub mod mnemonics;

use std::{
    collections::{BTreeMap, HashMap},
    mem,
};

pub use code_context::*;

use super::{
    ast::{self},
    stack::StackManager,
    variables::{StackLocation, ValueLocation, ValueType, Variable},
};
use mnemonics::*;

mod code_context;
mod stdlib;

pub fn build_code_context(
    block: &ast::Block,
    variables: &BTreeMap<String, Variable>,
    scopes: &HashMap<String, Vec<String>>,
    image_base: u64,
) -> CodeContext {
    let mut text_builder = TextBuilder::new(variables, scopes, image_base);
    text_builder.visit_ast(block);
    text_builder.get_code_context()
}

pub struct TextBuilder {
    code_context: CodeContext,
    variables: BTreeMap<String, Variable>,
    scopes: HashMap<String, Vec<String>>,
    stack_manager: StackManager,
}

impl TextBuilder {
    pub fn new(
        variables: &BTreeMap<String, Variable>,
        scopes: &HashMap<String, Vec<String>>,
        image_base: u64,
    ) -> Self {
        TextBuilder {
            code_context: CodeContext::new(image_base),
            variables: variables.clone(),
            scopes: scopes.clone(),
            stack_manager: StackManager::new(),
        }
    }

    pub fn get_code_context(&self) -> CodeContext {
        self.code_context.clone()
    }

    fn visit_ast(&mut self, block: &ast::Block) {
        self.stack_manager.init_stack();

        let call = self.call("main");
        self.code_context.add_slice(&call);
        stdlib::exit(&mut self.code_context, 0);

        self.visit_block(block);
    }

    fn visit_block(&mut self, block: &ast::Block) {
        self.stack_manager.init_stack();

        let mnemonics = self.allocate_stack(&block);
        self.code_context.add_slice(&mnemonics);
        block.stmts.iter().for_each(|stmt| {
            self.visit_statement(stmt, &block.scope);
        });
        self.code_context.add_slice(&self.stack_manager.free());
    }

    fn visit_statement(&mut self, statement: &ast::Statement, scope: &str) {
        // println!("{}: {:#?}", statement_n, &statement);
        match statement {
            ast::Statement::FuncDeclaration(func_decl) => {
                self.visit_func_declaration(func_decl);
            }
            ast::Statement::Expression(expr) => {
                self.visit_expression(expr, scope);
            }
            ast::Statement::VarDeclaration(_) => (),
            ast::Statement::Assignment(assign) => {
                self.visit_assignment(assign, scope);
            }
            ast::Statement::Block(stmts) => (),
            ast::Statement::Loop(l) => self.visit_loop(l, scope),
            ast::Statement::ControlFlow(_) => (),
        };
    }

    fn visit_func_declaration(&mut self, func_decl: &ast::FuncDeclaration) {
        let ast::FuncDeclaration {
            name,
            args,
            return_type,
            body,
        } = func_decl;

        self.code_context.set_label(name.value.clone());
        self.code_context
            .add_slice(&self.stack_manager.init_function_stack());

        self.code_context.add_slice(
            &self
                .stack_manager
                .push_registers(&abi::ARG_REGISTERS[..args.len()]),
        );

        for (i, arg) in args.iter().enumerate() {
            dbg!(&func_decl.name.value, &arg.name.value);
            let variable = self
                .get_variable_mut(&func_decl.body.scope, &arg.name.value)
                .unwrap();
            variable.value_loc = ValueLocation::Stack(StackLocation::Function(
                (mem::size_of::<u64>() * (i + 1)) as u64,
            ));
        }

        self.visit_block(body);

        self.code_context.add_slice(
            &self
                .stack_manager
                .pop_registers(&abi::ARG_REGISTERS[..args.len()]),
        );

        self.code_context
            .add_slice(&self.stack_manager.free_function_stack());
        self.code_context.add(RET.no_op());
    }

    fn visit_expression(&mut self, expr: &ast::Expression, scope: &str) {
        match expr {
            ast::Expression::Call(call) => {
                self.visit_call(call, scope);
            }
            _ => (),
        }
    }

    fn visit_assignment(&mut self, assign: &ast::Assignment, scope: &str) {
        let ast::Assignment {
            variable_name: id,
            rhs,
        } = assign;

        let expr = match rhs {
            ast::RhsExpression::Expression(expr) => expr,
            ast::RhsExpression::Block(block) => todo!(),
        };

        match expr {
            ast::Expression::Literal(lit) => {
                let variable = self
                    .get_variable(&scope, &id.value)
                    .unwrap_or_else(|| panic!("undefined variable: {}", id.value))
                    .clone();
                if matches!(variable.value_loc, ValueLocation::DataSection(_)) {
                    panic!("Cannot assign to const data: {variable:#?}");
                }
                let lit_data_type: ValueType = lit.clone().into();
                if std::mem::discriminant(&variable.value_type)
                    != std::mem::discriminant(&lit_data_type)
                {
                    panic!(
                        "Cannot assign {:?} to {:?}",
                        variable.value_type, lit_data_type
                    );
                }
                self.code_context.add_slice(&match lit {
                    ast::Literal::String(s) => {
                        self.stack_manager.push_list(&str_to_u64(s), s.len())
                    }
                    ast::Literal::Integer(n) => self.stack_manager.push(n.value as u64),
                });

                let value_loc = self.stack_manager.function_stack_size();

                let mut value_size = variable.value_size;
                let remainder = value_size % 8;
                if remainder != 0 {
                    value_size += 8 - remainder;
                }

                let variable = self
                    .get_variable_mut(&scope, &id.value)
                    .unwrap_or_else(|| panic!("undefined variable: {}", id.value));
                variable.value_size = value_size;
                variable.value_loc = ValueLocation::Stack(StackLocation::Block(value_loc as u64));
            }
            ast::Expression::Ident(_) => todo!(),
            ast::Expression::Call(_) => todo!(),
            ast::Expression::Unary(unary_operation) => todo!(),
            ast::Expression::Binary(binary_operation) => todo!(),
        };
    }

    fn visit_call(&mut self, call: &ast::Call, scope: &str) {
        if call.func_name.value == "print" {
            let (id, reference) = match call.args.first() {
                Some(ast::Expression::Ident(id)) => (id, false),
                Some(ast::Expression::Unary(unary)) => match unary {
                    ast::UnaryOperation::Ref(id) => match id.as_ref() {
                        ast::Expression::Ident(id) => (id, true),
                        _ => panic!("Function print expects an identifier"),
                    },
                    _ => panic!("Function print expects a reference"),
                },
                _ => panic!("Function print expects one argument"),
            };

            let mut variable = self
                .get_variable(&scope, &id.value)
                .unwrap_or_else(|| panic!("undefined variable: {}::{}", &scope, id.value))
                .clone();

            variable.reference = reference;

            match variable.value_type {
                ValueType::String(_) => {
                    let args = vec![variable.clone()];

                    abi::push_args(
                        &mut self.code_context,
                        &mut self.stack_manager,
                        args.as_slice(),
                    );

                    stdlib::print(&mut self.code_context, variable);

                    abi::pop_args(&mut self.code_context, &mut self.stack_manager, args.len());
                }
                ValueType::Int(n) => {
                    let mut format = self
                        .variables
                        .get("global::__printf_d_arg")
                        .unwrap_or_else(|| panic!("undefined variable: global::__printf_d_arg"))
                        .clone();

                    format.reference = true;

                    let args = &[format, variable.clone()];

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
            let args: Vec<Variable> = call
                .args
                .iter()
                .flat_map(|expr| {
                    let (id, reference) = match expr {
                        ast::Expression::Ident(id) => (id, false),
                        ast::Expression::Unary(unary) => match unary {
                            ast::UnaryOperation::Ref(id) => match id.as_ref() {
                                ast::Expression::Ident(id) => (id, true),
                                _ => panic!("Function print expects an identifier"),
                            },
                            _ => panic!("Function print expects a reference"),
                        },

                        _ => todo!(),
                    };
                    let mut variable = self
                        .get_variable(&scope, &id.value)
                        .unwrap_or_else(|| panic!("undefined symbol: {}", id.value))
                        .clone();

                    variable.reference = reference;

                    vec![variable]
                })
                .collect();

            abi::push_args(
                &mut self.code_context,
                &mut self.stack_manager,
                args.as_slice(),
            );

            let call_code = self.call(&call.func_name.value);
            self.code_context.add_slice(&call_code);
            abi::pop_args(&mut self.code_context, &mut self.stack_manager, args.len());
        }
    }

    fn visit_loop(&mut self, l: &ast::Loop, scope: &str) {
        self.stack_manager.init_stack();

        let block = &l.body;

        let mnemonics = self.allocate_stack(&block);
        self.code_context.add_slice(&mnemonics);
        let offset = self.code_context.get_code_size();

        block.stmts.iter().for_each(|stmt| {
            self.visit_statement(stmt, &block.scope);
        });

        let counter = self
            .get_variable(&l.body.scope, &l.var.value)
            .unwrap_or_else(|| panic!("undefined variable: {}::{}", l.body.scope, l.var.value));

        let value_loc: u32 = counter.value_loc.clone().into();

        self.code_context
            .add(MOV.op1(register::RCX).op2(register::RBP));
        self.code_context.add(SUB.op1(register::RCX).op2(value_loc));
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

        self.code_context.add_slice(&self.stack_manager.free());
    }

    fn allocate_stack(&mut self, stmts: &ast::Block) -> Vec<Mnemonic> {
        let ids: Vec<String> = match self.scopes.get(&stmts.scope) {
            Some(symbols) => symbols.clone(),
            None => return vec![],
        };

        dbg!(&ids);

        let mut code = vec![];
        for id in ids.iter() {
            let data = self.variables.get(id).unwrap();

            code.extend(match &data.value_loc {
                ValueLocation::Stack(stack_loc) => match stack_loc {
                    StackLocation::Block(_) => {
                        let code = match &data.value_type {
                            ValueType::String(s) => {
                                self.stack_manager.push_list(&str_to_u64(s), s.len())
                            }
                            ValueType::Int(i) => self.stack_manager.push(*i as u64),
                        };
                        dbg!(self.stack_manager.function_stack_size());
                        self.variables.get_mut(id).unwrap().value_loc = ValueLocation::Stack(
                            StackLocation::Block(self.stack_manager.function_stack_size() as u64),
                        );
                        code
                    }
                    StackLocation::Function(_) => vec![],
                },
                ValueLocation::DataSection(_) => vec![],
            });
        }
        // dbg!(self.stack_manager.get_top());
        code
    }

    fn get_variable(&self, scope: &str, id: &str) -> Option<&Variable> {
        let id_path = format!("{}::{}", scope, id);
        // dbg!(&scope, &id);

        self.variables.get(&id_path).or_else(|| {
            let (parent_scope, _) = scope.rsplit_once("::").unwrap();
            self.get_variable(parent_scope, id)
        })
    }

    fn get_variable_mut(&mut self, scope: &str, id: &str) -> Option<&mut Variable> {
        let id = format!("{}::{}", scope, id);
        // dbg!(&scope, &id);

        self.variables.get_mut(&id)
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
