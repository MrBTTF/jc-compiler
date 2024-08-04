use crate::emitter::ast::{Ident, Literal, Loop};
use std::{
    borrow::BorrowMut,
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
    rc::Rc,
};

use super::{
    ast::{self, DeclarationType},
    StatementList,
};

#[derive(Default, Debug, Clone)]
pub struct DataRef {
    pub symbol: String,
    pub offset: usize,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct Data {
    pub symbol: String,
    pub lit: ast::Literal,
    pub data_loc: u64,
    pub decl_type: ast::DeclarationType,
}

impl Data {
    pub fn new(
        symbol: &str,
        lit: ast::Literal,
        data_loc: u64,
        assign_type: ast::DeclarationType,
    ) -> Data {
        Data {
            symbol: symbol.to_string(),
            lit,
            data_loc,
            decl_type: assign_type,
        }
    }
}

#[derive(Default, Debug)]
pub struct DataBuilder {
    pub symbol_data: HashMap<ast::Ident, Data>,
    pub scope_symbols: HashMap<usize, Vec<ast::Ident>>,
    data_section: Vec<usize>,
    stack: Vec<usize>,
}

impl DataBuilder {
    pub fn visit_ast(&mut self, statement_list: &ast::StatementList) {
        let lit = Literal::String("%d\0".to_string());
        let lit_size = lit.len(); //TODO
        self.data_section.push(lit.len());
        let id = Ident {
            value: "__printf_d_arg".to_string(),
        };
        self.symbol_data.insert(
            id.clone(),
            Data::new(&id.value, lit, 0 as u64, DeclarationType::Const),
        );
        self.add_to_scope(&statement_list, vec![id.clone()]);

        self.visit_statement_list(statement_list);
    }

    pub fn visit_statement_list(&mut self, statement_list: &ast::StatementList) {
        let mut stack = vec![];
        statement_list.stmts.iter().for_each(|stmt| {
            let ids = self.visit_statement(stmt, &mut stack);
            self.add_to_scope(statement_list, ids);
        });
    }

    fn visit_statement(
        &mut self,
        statement: &ast::Statement,
        stack: &mut Vec<usize>,
    ) -> Vec<ast::Ident> {
        let mut ids = vec![];
        match statement {
            ast::Statement::Expression(expr) => match expr {
                ast::Expression::Loop(l) => self.visit_loop(l),
                _ => (),
            },
            ast::Statement::Declaration(ast::Declaration(id, expr, assign_type)) => match expr {
                ast::Expression::Literal(lit) => {
                    let data_loc: usize = match assign_type {
                        DeclarationType::Let => {
                            let mut data_size = lit.len();
                            let remainder = data_size % 8;
                            if remainder != 0 {
                                data_size += 8 - remainder;
                            }
                            stack.push(data_size);
                            stack.iter().sum()
                        }
                        DeclarationType::Const => {
                            let data_loc = self.data_section.iter().sum();
                            self.data_section.push(lit.len());
                            data_loc
                        }
                    };
                    let data: Data =
                        Data::new(&id.value, lit.clone(), data_loc as u64, *assign_type);
                    self.symbol_data.insert(id.clone(), data.clone());
                    ids.push(id.clone());
                }
                _ => todo!(),
            },
            ast::Statement::FuncDefinition(ast::FuncDefinition(_, _, _, stmt_list)) => {
                self.visit_statement_list(stmt_list)
            }
            ast::Statement::Scope(stmt_list) => self.visit_statement_list(stmt_list),
            ast::Statement::Assignment(ast::Assignment(_id, _expr)) => (),
            ast::Statement::ControlFlow(_) => (),
        }
        ids
    }

    // fn visit_declaration(&mut self, statement: &ast::Statement) {}
    fn visit_loop(&mut self, l: &Loop) {
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
            self.stack.push(data_size);
            self.stack.iter().sum()
        };

        let data = Data::new(&id.value, lit, data_loc as u64, ast::DeclarationType::Let);
        self.symbol_data.insert(id.clone(), data.clone());
        self.add_to_scope(&l.body, vec![id.clone()]);
    }

    fn add_to_scope(&mut self, stmts: &StatementList, id: Vec<ast::Ident>) {
        match self.scope_symbols.entry(stmts.id) {
            Entry::Vacant(v) => {
                v.insert(id);
            }
            Entry::Occupied(mut o) => {
                o.get_mut().extend(id);
            }
        }
    }
}
