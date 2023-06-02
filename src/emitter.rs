use std::{collections::HashMap, fs, io::Write};

use self::{amd64::*, ast::Visitor, elf::structs::*, stdlib::print};

mod amd64;
pub mod ast;
pub mod elf;
pub mod stdlib;

pub fn build_elf(ast: &ast::StatementList) {
    let mut data_builder = DataBuilder::default();
    data_builder.visit_statement_list(ast);
    let mut elf_emitter = ElfEmitter {
        literals: data_builder.literals.clone(),
    };
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

#[derive(Default)]
pub struct DataBuilder {
    literals: HashMap<ast::Ident, (ast::Literal, u32)>,
    length: u32,
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
            ast::Statement::Assignment(id, expr) => match expr {
                ast::Expression::Literal(lit) => {
                    let data_loc = self.length;
                    self.literals
                        .insert(id.clone(), (lit.clone(), data_loc as u32));
                    self.length += match lit.clone() {
                        ast::Literal::String(string) => string.len(),
                        _ => 0,
                    } as u32;
                }
                _ => (),
            },
        }
    }
}

pub struct ElfEmitter {
    literals: HashMap<ast::Ident, (ast::Literal, u32)>,
}

impl ElfEmitter {
    fn visit_call(&mut self, id: &ast::Ident, expr: &ast::Expression) -> Vec<u8> {
        let (lit, data_loc) = match expr {
            ast::Expression::Literal(ast::Literal::Ident(id)) => self
                .literals
                .get(&id)
                .expect(&format!("undefined variable: {}", id.value)),
            _ => todo!(),
        };

        if let ast::Literal::String(string) = lit {
            if id.value == "print" {
                return print(*data_loc, string.len());
            }
        }

        panic!("no such function {}", id.value)
    }
}

impl Visitor<Vec<u8>> for ElfEmitter {
    fn visit_statement_list(&mut self, statement_list: &ast::StatementList) -> Vec<u8> {
        let mut result = statement_list.0.iter().fold(vec![], |mut result, stmt| {
            result.extend(self.visit_statement(stmt));
            result
        });
        result.extend_from_slice(
            &[
                Mov32::build(Register::Ebx, 0x0),
                Mov32::build(Register::Eax, 0x1),
                SysCall::build(),
            ]
            .concat(),
        );
        result
    }

    fn visit_statement(&mut self, statement: &ast::Statement) -> Vec<u8> {
        match statement {
            ast::Statement::Expression(expr) => match expr {
                ast::Expression::Call(id, expr) => self.visit_call(id, expr),

                _ => vec![],
            },
            ast::Statement::Assignment(id, expr) => vec![],
        }
    }

    fn visit_expression(&mut self, expr: &ast::Expression) -> Vec<u8> {
        todo!()
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
