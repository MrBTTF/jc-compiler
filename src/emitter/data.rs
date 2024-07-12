use crate::emitter::ast::{Ident, Literal, Loop};
use std::collections::BTreeMap;

use super::ast::{self, DeclarationType};

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
    pub variables: BTreeMap<ast::Ident, Data>,
    pub data_ordered: Vec<ast::Ident>,
    data_section: Vec<usize>,
    stack: Vec<usize>,
    current_line: usize,
}

impl DataBuilder {
    pub fn visit_statement_list(&mut self, statement_list: &ast::StatementList) {
        let lit = Literal::String("%d\0".to_string());
        let lit_size = lit.len(); //TODO
        self.data_section.push(lit.len());
        let id = Ident {
            value: "__printf_d_arg".to_string(),
        };
        self.variables.insert(
            id.clone(),
            Data::new(&id.value, lit, 0 as u64, DeclarationType::Const),
        );
        self.data_ordered.push(id.clone());

        statement_list
            .0
            .iter()
            .for_each(|stmt| self.visit_statement(stmt));
    }

    fn visit_statement(&mut self, statement: &ast::Statement) {
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
                            self.stack.push(data_size);
                            self.stack.iter().sum()
                        }
                        DeclarationType::Const => {
                            let data_loc = self.data_section.iter().sum();
                            self.data_section.push(lit.len());
                            data_loc
                        }
                    };
                    let data: Data =
                        Data::new(&id.value, lit.clone(), data_loc as u64, *assign_type);
                    self.variables.insert(id.clone(), data.clone());
                    self.data_ordered.push(id.clone());
                }
                _ => todo!(),
            },
            ast::Statement::FuncDeclaration(_) => (),
            ast::Statement::Assignment(ast::Assignment(_id, _expr)) => (),
            ast::Statement::ControlFlow(_) => (),
        }
        self.current_line += 1;
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
        self.variables.insert(id.clone(), data.clone());
        self.data_ordered.push(id.clone());
    }
}
