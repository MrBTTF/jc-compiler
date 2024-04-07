use std::{collections::HashMap, ffi::CString};

use super::{
    ast::{self, AssignmentType},
    elf::sections::{DWord, DATA_SECTION_ADDRESS_START},
};

#[derive(Default, Debug)]
pub struct DataRef {
    pub offset: usize,
    pub ref_len: usize,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct Data {
    pub lit: ast::Literal,
    data_loc: DWord,
    pub assign_type: ast::AssignmentType,
}
impl Data {
    pub fn new(lit: ast::Literal, data_loc: DWord, assign_type: ast::AssignmentType) -> Data {
        Data {
            lit,
            data_loc,
            assign_type,
        }
    }

    pub fn data_loc(&self) -> u64 {
        match self.assign_type {
            AssignmentType::Let => self.data_loc,
            AssignmentType::Const => DATA_SECTION_ADDRESS_START + self.data_loc,
        }
    }
}

#[derive(Default, Debug)]
pub struct DataBuilder {
    pub variables: HashMap<ast::Ident, Data>,
    data_section: Vec<usize>,
    stack: Vec<usize>,
    current_line: usize,
}

impl DataBuilder {
    pub fn visit_statement_list(&mut self, statement_list: &ast::StatementList) {
        statement_list
            .0
            .iter()
            .for_each(|stmt| self.visit_statement(stmt));
    }

    fn visit_statement(&mut self, statement: &ast::Statement) {
        match statement {
            ast::Statement::Expression(_) => (),
            ast::Statement::Assignment(ast::Assignment(id, expr, assign_type)) => match expr {
                ast::Expression::Literal(lit) => {
                    let data_loc: usize = match assign_type {
                        AssignmentType::Let => {
                            let mut data_size = lit.len();
                            let remainder = data_size % 8;
                            if remainder != 0 {
                                data_size += 8 - remainder;
                            }
                            self.stack.push(data_size);
                            self.stack.iter().sum()
                        }
                        AssignmentType::Const => {
                            let data_loc = self.data_section.iter().sum();
                            self.data_section.push(lit.len());
                            data_loc
                        }
                    };
                    self.variables.insert(
                        id.clone(),
                        Data::new(lit.clone(), data_loc as DWord, *assign_type),
                    );
                }
                _ => todo!(),
            },
        }
        self.current_line += 1;
    }

    // fn visit_assignment(&mut self, statement: &ast::Statement) {}
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
                    assign_type,
                },
            )| {
                match assign_type {
                    ast::AssignmentType::Let => None,
                    ast::AssignmentType::Const => Some((*data_loc, id.clone(), lit.clone())),
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
