use std::{collections::HashMap, fs, io::Write, mem};

use self::{
    amd64::*,
    ast::{Assignment, AssignmentType, Visitor},
    elf::structs::*,
    stdlib::print,
};

mod abi;
mod amd64;
pub mod ast;
pub mod elf;
pub mod stdlib;

pub fn build_elf(ast: &ast::StatementList) {
    let mut data_builder = DataBuilder::default();
    data_builder.visit_statement_list(ast);

    let mut elf_emitter = ElfEmitter::new(&data_builder);

    let text_header = &elf_emitter.visit_statement_list(ast);

    let data_header = &build_data_section(data_builder.literals.clone());
    // let text_header = &build_text_section();
    let shstrtab_header = &build_shstrtab_section();
    let program_headers = &build_program_headers(text_header.len(), data_header.len());
    let header = &build_header(
        data_header.len(),
        program_headers.len() + text_header.len() + data_header.len() + shstrtab_header.len(),
    )
    .as_slice()
    .to_owned();

    let mut file = fs::File::create("bin/hello").unwrap();
    file.write_all(header).unwrap();
    file.write_all(program_headers).unwrap();

    file.write_all(data_header).unwrap();
    file.write_all(text_header).unwrap();
    file.write_all(shstrtab_header).unwrap();

    file.write_all(&build_section_headers(
        text_header.len(),
        data_header.len(),
        shstrtab_header.len(),
    ))
    .unwrap();
}

#[derive(Clone)]
pub struct Data {
    pub lit: ast::Literal,
    pub data_loc: DWord,
    pub assign_type: ast::AssignmentType,
}
impl Data {
    fn new(lit: ast::Literal, data_loc: DWord, assign_type: ast::AssignmentType) -> Data {
        Data {
            lit,
            data_loc,
            assign_type,
        }
    }
}

#[derive(Default)]
pub struct DataBuilder {
    literals: HashMap<ast::Ident, Data>,
    data_section: Vec<usize>,
    stack: Vec<usize>,
    // length: u32,
}

impl DataBuilder {
    fn visit_statement_list(&mut self, statement_list: &ast::StatementList) {
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
                    let data_loc = match assign_type {
                        AssignmentType::Let => match lit.clone() {
                            ast::Literal::String(s) => {
                                let mut data_size = s.len();
                                data_size += 4 - (data_size % 4);
                                self.stack.push(data_size);
                                self.stack.iter().sum()
                            }
                            _ => 0,
                        },
                        AssignmentType::Const => match lit.clone() {
                            ast::Literal::String(s) => {
                                let data_loc = self.data_section.iter().sum();
                                self.data_section.push(s.len());
                                data_loc
                            }
                            _ => 0,
                        },
                    };

                    self.literals.insert(
                        id.clone(),
                        Data::new(lit.clone(), data_loc as DWord, *assign_type),
                    );
                }
                _ => (),
            },
        }
    }

    fn visit_assignment(&mut self, statement: &ast::Statement) {}
}

pub struct ElfEmitter {
    literals: HashMap<ast::Ident, Data>,
    entry_point: DWord,
}

impl ElfEmitter {
    fn new(data_builder: &DataBuilder) -> Self {
        let entry_point: DWord = (mem::size_of::<ELFHeader>()
            + mem::size_of::<ProgramHeader>() * 3)
            .try_into()
            .unwrap();
        ElfEmitter {
            literals: data_builder.literals.clone(),
            entry_point: VIRTUAL_ADDRESS_START + entry_point,
        }
    }

    fn visit_call(&mut self, id: &ast::Ident, expr: &ast::Expression) -> Vec<u8> {
        let Data {
            lit,
            data_loc,
            assign_type,
        } = match expr {
            ast::Expression::Literal(ast::Literal::Ident(id)) => self
                .literals
                .get(id)
                .unwrap_or_else(|| panic!("undefined variable: {}", id.value)),
            _ => todo!(),
        };

        if id.value == "print" {
            let args = &[match assign_type {
                AssignmentType::Let => abi::Arg::Stack(*data_loc as i64),
                AssignmentType::Const => abi::Arg::Data((self.entry_point + *data_loc) as i64),
            }];
            return [
                abi::push_args(args),
                print(lit.len()),
                abi::pop_args(args.len()),
            ]
            .concat();
        }

        panic!("no such function {}", id.value)
    }
}

impl Visitor<Vec<u8>> for ElfEmitter {
    fn visit_statement_list(&mut self, statement_list: &ast::StatementList) -> Vec<u8> {
        let mut result = vec![];
        result.extend(Mov64rr::build(Register::Bp, Register::Sp));
        result.extend(statement_list.0.iter().fold(vec![], |mut result, stmt| {
            result.extend(self.visit_statement(stmt));
            result
        }));
        result.extend_from_slice(&stdlib::exit(0));
        result
    }

    fn visit_statement(&mut self, statement: &ast::Statement) -> Vec<u8> {
        match statement {
            ast::Statement::Expression(expr) => self.visit_expression(expr),
            ast::Statement::Assignment(Assignment(id, expr, AssignmentType::Let)) => match expr {
                ast::Expression::Literal(ast::Literal::String(s)) => {
                    let pushes: Vec<_> =
                        s.as_bytes()
                            .chunks(8)
                            .rev()
                            .fold(vec![], |mut acc, substr| {
                                let mut value: u64 = 0;
                                for (i, c) in substr.iter().enumerate() {
                                    value += (*c as u64) << (8 * i)
                                }
                                acc.extend(Mov64::build(Register::Ax, value as i64));
                                acc.extend(Push::build(Register::Ax));
                                acc
                            });
                    pushes
                }
                _ => vec![],
            },
            ast::Statement::Assignment(Assignment(_, _, AssignmentType::Const)) => vec![],
        }
    }

    fn visit_expression(&mut self, expr: &ast::Expression) -> Vec<u8> {
        match expr {
            ast::Expression::Call(id, expr) => self.visit_call(id, expr),

            _ => vec![],
        }
    }

    fn visit_literal(&mut self, literal: &ast::Literal) -> Vec<u8> {
        match literal {
            ast::Literal::Ident(_) => todo!(),
            ast::Literal::String(_) => todo!(),
            ast::Literal::Number(_) => todo!(),
        }
    }

    fn visit_ident(&mut self, ident: &ast::Ident) -> Vec<u8> {
        todo!()
    }

    fn visit_number(&mut self, number: &ast::Number) -> Vec<u8> {
        todo!()
    }
}
