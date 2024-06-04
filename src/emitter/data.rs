use std::{collections::{BTreeMap, HashMap}, ffi::CString, mem};
use crate::emitter::ast::{Ident, Literal, Loop};

use super::{
    ast::{self, DeclarationType},
    elf::sections::{DWord, DATA_SECTION_ADDRESS_START},
};

#[derive(Default, Debug, Clone)]
pub struct DataRef {
    pub offset: usize,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct Data {
    pub lit: ast::Literal,
    data_loc: DWord,
    pub decl_type: ast::DeclarationType,
}
impl Data {
    pub fn new(lit: ast::Literal, data_loc: DWord, assign_type: ast::DeclarationType) -> Data {
        Data {
            lit,
            data_loc,
            decl_type: assign_type,
        }
    }

    pub fn data_loc(&self) -> u64 {
        match self.decl_type {
            DeclarationType::Let => self.data_loc,
            DeclarationType::Const => DATA_SECTION_ADDRESS_START + self.data_loc,
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
        let lit_size = lit.len();
        self.data_section.push(lit.len());
        let id = Ident{
            value: "__printf_d_arg".to_string(),
        };
        self.variables.insert(
            id.clone(),
            Data::new(lit, lit_size as DWord, DeclarationType::Const),
        );
        self.data_ordered.push(id.clone());

        statement_list
            .0
            .iter()
            .for_each(|stmt| self.visit_statement(stmt));
    }

    fn visit_statement(&mut self, statement: &ast::Statement) {
        match statement {
            ast::Statement::Expression(expr) => {
                match expr {
                    ast::Expression::Loop(l) => self.visit_loop(l),
                    _ => ()
                }
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
                    let data = Data::new(lit.clone(), data_loc as DWord, *assign_type);
                    self.variables.insert(
                        id.clone(),
                        data.clone(),
                    );
                    self.data_ordered.push(id.clone());

                }
                _ => todo!(),
            },
            ast::Statement::Assignment(ast::Assignment(id, expr)) => (),
        }
        self.current_line += 1;
    }

    // fn visit_declaration(&mut self, statement: &ast::Statement) {}
    fn visit_loop(&mut self, l: &Loop) {
       let id = &l.var;
       let lit = Literal::Number(ast::Number{value: l.start as i64});
       let data_loc: usize = {
           let mut data_size = lit.len();
           let remainder = data_size % 8;
           if remainder != 0 {
               data_size += 8 - remainder;
           }
           self.stack.push(data_size);
           self.stack.iter().sum()
       };

        let data=  Data::new(lit, data_loc as DWord, ast::DeclarationType::Let);
        self.variables.insert(
            id.clone(),
            data.clone(),
        );
        self.data_ordered.push(id.clone());
    }
}

pub fn build_data_section(literals: HashMap<ast::Ident, Data>) -> Vec<u8> {
    let mut literals: Vec<_> = literals
        .iter()
        .filter_map(
            |(
                id,
                Data {
                    lit,
                    data_loc,
                    decl_type: assign_type,
                },
            )| {
                match assign_type {
                    ast::DeclarationType::Let => None,
                    ast::DeclarationType::Const => Some((*data_loc, id.clone(), lit.clone())),
                }
            },
        )
        .collect();
    literals.sort_by_key(|(data_loc, _, _)| *data_loc);
    literals
        .iter()
        .fold(vec![], |mut acc, (_, _, lit)| match lit {
            ast::Literal::String(string) => {
                acc.extend(CString::new(string.clone()).unwrap().into_bytes());
                acc
            }
            ast::Literal::Number(n) => {
                acc.extend(n.value.to_le_bytes().to_vec());
                acc
            }
        })
}
