use crate::emitter::ast::{Ident, Literal, Loop};
use std::{
    borrow::BorrowMut,
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
    mem,
    rc::Rc,
};

use super::{
    ast::{self, DeclarationType},
    Number, StatementList,
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
            Literal::Number(i) => DataType::Int(i.value),
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
    pub decl_type: ast::DeclarationType,
}

impl Data {
    pub fn new(
        symbol: &str,
        data_type: DataType,
        reference: bool,
        data_loc: u64,
        assign_type: ast::DeclarationType,
    ) -> Data {
        let data_size = match &data_type {
            DataType::String(s) => match assign_type {
                DeclarationType::Let => s.len() + mem::size_of::<u64>(),
                DeclarationType::Const => s.len(),
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
    statement_list: &ast::StatementList,
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
    pub fn visit_ast(&mut self, statement_list: &ast::StatementList) {
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
                DeclarationType::Const,
            ),
        );
        self.add_to_scope(&statement_list.id, vec![printf_d_arg.to_string()]);

        self.visit_statement_list(statement_list);
    }

    pub fn visit_statement_list(&mut self, statement_list: &ast::StatementList) {
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
            ast::Statement::Expression(expr) => match expr {
                ast::Expression::Loop(l) => self.visit_loop(l, stack),
                _ => (),
            },
            ast::Statement::Declaration(ast::Declaration(id, expr, assign_type)) => match expr {
                ast::Expression::Literal(lit) => {
                    let data_loc: usize = match assign_type {
                        DeclarationType::Let => get_position_on_stack(stack, lit),
                        DeclarationType::Const => get_position_on_data(&mut self.data_section, lit),
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
            },
            ast::Statement::FuncDefinition(ast::FuncDefinition(f, args, _, stmt_list)) => {
                for arg in args {
                    let data_loc: usize = {
                        stack.push(mem::size_of::<u64>());
                        stack.iter().sum()
                    };
                    let (lit, _ref) = match &arg._type {
                        ast::Type::String => (Literal::String("".to_string()), false),
                        ast::Type::Number => (Literal::Number(Number { value: 0 }), false),
                        ast::Type::Ref(t) => (
                            match **t {
                                ast::Type::String => Literal::String("".to_string()),
                                ast::Type::Number => Literal::Number(Number { value: 0 }),
                                ast::Type::Ref(_) => todo!(),
                            },
                            true,
                        ),
                    };

                    let id = format!("{}::{}", f.value, &arg.name.value);
                    let data =
                        Data::new(&id, lit.into(), _ref, data_loc as u64, DeclarationType::Let);
                    self.symbol_data.insert(id.clone(), data.clone());
                    ids.push(id.clone());
                }
                dbg!(&stack);
                self.visit_statement_list(stmt_list)
            }
            ast::Statement::Scope(stmt_list) => self.visit_statement_list(stmt_list),
            ast::Statement::Assignment(ast::Assignment(_id, _expr)) => (),
            ast::Statement::ControlFlow(_) => (),
        }
        ids
    }

    // fn visit_declaration(&mut self, statement: &ast::Statement) {}
    fn visit_loop(&mut self, l: &Loop, stack: &mut Vec<usize>) {
        let mut stack = vec![];
        let id = &l.var;
        let lit = Literal::Number(ast::Number {
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
            ast::DeclarationType::Let,
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
