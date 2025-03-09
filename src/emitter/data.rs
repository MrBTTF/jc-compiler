use crate::emitter::ast::{Ident, Literal, Loop};
use std::{
    borrow::BorrowMut,
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
    mem,
    rc::Rc,
};

use super::{
    ast::{self, VarDeclarationType},
    Block, Integer,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataType {
    String(String),
    Int(i64),
}

impl From<ast::Literal> for DataType {
    fn from(value: ast::Literal) -> Self {
        match value {
            Literal::String(s) => DataType::String(s),
            Literal::Integer(i) => DataType::Int(i.value),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct DataRef {
    pub symbol: String,
    pub offset: usize,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct Data {
    pub symbol: String,
    pub data_type: DataType,
    pub data_size: usize,
    pub reference: bool,
    pub data_loc: u64,
    pub decl_type: ast::VarDeclarationType,
}

impl Data {
    pub fn new(
        symbol: &str,
        data_type: DataType,
        reference: bool,
        data_loc: u64,
        assign_type: ast::VarDeclarationType,
    ) -> Data {
        let data_size = match &data_type {
            DataType::String(s) => match assign_type {
                VarDeclarationType::Let => s.len() + mem::size_of::<u64>(),
                VarDeclarationType::Const => s.len(),
            },
            DataType::Int(_) => mem::size_of::<i64>(),
        };
        Data {
            symbol: symbol.to_string(),
            data_type,
            data_size,
            reference,
            data_loc,
            decl_type: assign_type,
        }
    }

    pub fn set_data_type(&mut self, data_type: DataType) {
        self.data_type = data_type;
        let mut data_size = match &self.data_type {
            DataType::String(s) => s.len() + mem::size_of::<u64>(),
            DataType::Int(_) => mem::size_of::<i64>(),
        };
        let remainder = data_size % 8;
        if remainder != 0 {
            data_size += 8 - remainder;
        }
        let diff = data_size as i64 - self.data_size as i64;
        dbg!(self.data_loc, &self.data_type, data_size, self.data_size);
        self.data_loc = self.data_loc.wrapping_add_signed(diff);

        dbg!(self.data_loc);
        self.data_size = data_size;
    }

    pub fn as_vec(&self) -> Vec<u8> {
        match &self.data_type {
            DataType::String(s) => [s.as_bytes().to_vec(), vec![0]].concat(),
            DataType::Int(i) => i.to_le_bytes().to_vec(),
        }
    }
}

pub fn build_symbol_data(
    statement_list: &ast::Block,
) -> (HashMap<String, Data>, HashMap<String, Vec<String>>) {
    let mut data_builder = DataBuilder::default();
    data_builder.visit_ast(statement_list);
    (data_builder.symbol_data, data_builder.scope_symbols)
}

#[derive(Default, Debug)]
pub struct DataBuilder {
    pub symbol_data: HashMap<String, Data>,
    pub scope_symbols: HashMap<String, Vec<String>>,
    data_section: Vec<usize>,
    stack: Vec<usize>,
}

impl DataBuilder {
    pub fn visit_ast(&mut self, statement_list: &ast::Block) {
        let number_fmt = "%d\0".to_string();
        self.data_section.push(number_fmt.len());
        let data_type = DataType::String(number_fmt);
        let printf_d_arg = "global::__printf_d_arg";
        self.symbol_data.insert(
            printf_d_arg.to_string(),
            Data::new(
                &printf_d_arg,
                data_type,
                false,
                0 as u64,
                VarDeclarationType::Const,
            ),
        );
        self.add_to_scope(&statement_list.id, vec![printf_d_arg.to_string()]);

        self.visit_statement_list(statement_list);
    }

    pub fn visit_statement_list(&mut self, statement_list: &ast::Block) {
        let mut stack = vec![];
        statement_list.stmts.iter().for_each(|stmt| {
            let ids = self.visit_statement(stmt, &mut stack, &statement_list.id);
            self.add_to_scope(&statement_list.id, ids);
        });
    }

    fn visit_statement(
        &mut self,
        statement: &ast::Statement,
        stack: &mut Vec<usize>,
        scope: &str,
    ) -> Vec<String> {
        let mut ids = vec![];
        match statement {
            ast::Statement::Expression(expression) => (),
            ast::Statement::Loop(l) => self.visit_loop(l, stack),
            ast::Statement::VarDeclaration(ast::VarDeclaration {
                name: id,
                rhs: rhs,
                declarion_type: assign_type,
            }) => {
                let expr = match rhs {
                    ast::RhsExpression::Expression(expr) => expr,
                    ast::RhsExpression::Block(block) => todo!(),
                };

                match expr {
                    ast::Expression::Literal(lit) => {
                        let data_loc: usize = match assign_type {
                            VarDeclarationType::Let => get_position_on_stack(stack, lit),
                            VarDeclarationType::Const => {
                                get_position_on_data(&mut self.data_section, lit)
                            }
                        };
                        let id = format!("{}::{}", scope, &id.value);
                        let data = Data::new(
                            &id,
                            lit.clone().into(),
                            false,
                            data_loc as u64,
                            *assign_type,
                        );
                        self.symbol_data.insert(id.clone(), data.clone());
                        ids.push(id.clone());
                    }
                    _ => todo!(),
                }
            }
            ast::Statement::FuncDeclaration(ast::FuncDeclaration {
                name: f,
                args,
                return_type: _,
                body: stmt_list,
            }) => {
                for arg in args {
                    let data_loc: usize = {
                        stack.push(mem::size_of::<u64>());
                        stack.iter().sum()
                    };

                    let has_ref = !arg._type.modifiers.is_empty();
                    let lit = match arg._type.name {
                        ast::TypeName::String => Literal::String("".to_string()),
                        ast::TypeName::Int => Literal::Integer(Integer { value: 0 }),
                        ast::TypeName::Float => todo!(),
                        ast::TypeName::Bool => todo!(),
                        ast::TypeName::Unit => todo!(),
                    };

                    let id = format!("{}::{}", f.value, &arg.name.value);
                    let data = Data::new(
                        &id,
                        lit.into(),
                        has_ref,
                        data_loc as u64,
                        VarDeclarationType::Let,
                    );
                    self.symbol_data.insert(id.clone(), data.clone());
                    ids.push(id.clone());
                }
                dbg!(&stack);
                self.visit_statement_list(stmt_list)
            }
            ast::Statement::Block(stmt_list) => self.visit_statement_list(stmt_list),
            ast::Statement::Assignment(_) => (),
            ast::Statement::ControlFlow(_) => (),
        }
        ids
    }

    // fn visit_declaration(&mut self, statement: &ast::Statement) {}
    fn visit_loop(&mut self, l: &Loop, stack: &mut Vec<usize>) {
        let mut stack = vec![];
        let id = &l.var;
        let lit = Literal::Integer(ast::Integer {
            value: l.start as i64,
        });
        let data_loc: usize = {
            let mut data_size = lit.len();
            let remainder = data_size % 8;
            if remainder != 0 {
                data_size += 8 - remainder;
            }
            stack.push(data_size);
            stack.iter().sum()
        };
        let id = format!("{}::{}", l.body.id, &id.value);
        let data = Data::new(
            &id,
            lit.into(),
            false,
            data_loc as u64,
            ast::VarDeclarationType::Let,
        );
        self.symbol_data.insert(id.clone(), data.clone());
        self.add_to_scope(&l.body.id, vec![id.clone()]);
    }

    fn add_to_scope(&mut self, parent: &str, ids: Vec<String>) {
        match self.scope_symbols.entry(parent.to_string()) {
            Entry::Vacant(v) => {
                v.insert(ids);
            }
            Entry::Occupied(mut o) => {
                o.get_mut().extend(ids);
            }
        }
    }
}

fn get_position_on_stack(stack: &mut Vec<usize>, lit: &ast::Literal) -> usize {
    let mut data_size = lit.len();
    let remainder = data_size % 8;
    if remainder != 0 {
        data_size += 8 - remainder;
    }
    stack.push(data_size);

    if let ast::Literal::String(_) = lit {
        stack.push(mem::size_of::<u64>()); // length of string
    }
    stack.iter().sum()
}

fn get_position_on_data(data_section: &mut Vec<usize>, lit: &ast::Literal) -> usize {
    let data_loc = data_section.iter().sum();
    data_section.push(lit.len());
    data_loc
}
