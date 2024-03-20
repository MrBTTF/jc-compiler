pub mod defs;
pub mod sections;

use std::{collections::HashMap, fs, io::Write};

use self::{
    super::{amd64::*, ast, data::*, structs::*},
    sections::*,
};

use super::abi::linux as abi;
use super::stdlib::linux as stdlib;

pub fn build_elf(ast: &ast::StatementList) {
    let mut data_builder = DataBuilder::default();
    data_builder.visit_statement_list(ast);

    let mut elf_emitter = ElfEmitter::new(&data_builder);

    let instructions = elf_emitter.visit_statement_list(ast);
    // dbg!(&instructions);
    let text_header = instructions.to_bin();

    let data_header = &build_data_section(data_builder.variables.clone());
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
    file.write_all(&text_header).unwrap();
    file.write_all(shstrtab_header).unwrap();

    file.write_all(&build_section_headers(
        text_header.len(),
        data_header.len(),
        shstrtab_header.len(),
    ))
    .unwrap();
}

pub struct ElfEmitter {
    literals: HashMap<ast::Ident, Data>,
}

impl ElfEmitter {
    fn new(data_builder: &DataBuilder) -> Self {
        ElfEmitter {
            literals: data_builder.variables.clone(),
        }
    }

    fn visit_statement_list(&mut self, statement_list: &ast::StatementList) -> Instructions {
        let mut result: Instructions = vec![];
        result.push(Mov64rr::new(Register::Bp, Register::Sp));
        result.extend(statement_list.0.iter().fold(vec![], |mut result, stmt| {
            result.extend(self.visit_statement(stmt));
            result
        }));
        result.extend(stdlib::exit(0));
        result
    }

    fn visit_statement(&mut self, statement: &ast::Statement) -> Instructions {
        match statement {
            ast::Statement::Expression(expr) => self.visit_expression(expr),
            ast::Statement::Assignment(ast::Assignment(_, expr, ast::AssignmentType::Let)) => {
                match expr {
                    ast::Expression::Literal(ast::Literal::String(s)) => {
                        let pushes: Vec<_> = s.as_bytes().chunks(8).rev().fold(
                            vec![],
                            |mut acc: Instructions, substr| {
                                let mut value: u64 = 0;
                                for (i, c) in substr.iter().enumerate() {
                                    value += (*c as u64) << (8 * i)
                                }
                                acc.push(Mov64::new(Register::Ax, value as i64));
                                acc.push(Push::new(Register::Ax));
                                acc
                            },
                        );
                        pushes
                    }
                    ast::Expression::Literal(ast::Literal::Number(n)) => {
                        vec![Mov64::new(Register::Ax, n.value), Push::new(Register::Ax)]
                    }
                    _ => vec![],
                }
            }
            ast::Statement::Assignment(ast::Assignment(_, _, ast::AssignmentType::Const)) => vec![],
        }
    }

    fn visit_expression(&mut self, expr: &ast::Expression) -> Instructions {
        match expr {
            ast::Expression::Call(id, expr) => self.visit_call(id, expr),

            _ => vec![],
        }
    }

    fn visit_call(&mut self, id: &ast::Ident, expr: &ast::Expression) -> Instructions {
        let data = match expr {
            ast::Expression::Ident(id) => self
                .literals
                .get(id)
                .unwrap_or_else(|| panic!("undefined variable: {}", id.value)),
            _ => todo!(),
        };

        if id.value == "print" {
            let args = &[match data.assign_type {
                ast::AssignmentType::Let => abi::Arg::Stack(data.data_loc() as i64),
                ast::AssignmentType::Const => abi::Arg::Data(data.data_loc() as i64),
            }];
            let print_call = match data.lit {
                ast::Literal::String(_) => stdlib::print(data.lit.len()),
                ast::Literal::Number(_) => stdlib::printd(),
            };
            let mut result = vec![];
            result.extend(abi::push_args(args));
            result.extend(print_call);
            result.extend(abi::pop_args(args.len()));
            return result;
        }

        panic!("no such function {}", id.value)
    }

    // fn visit_literal(&mut self, literal: &ast::Literal) -> Vec<u8> {
    //     match literal {
    //         ast::Literal::String(_) => todo!(),
    //         ast::Literal::Number(_) => todo!(),
    //     }
    // }

    // fn visit_ident(&mut self, ident: &ast::Ident) -> Vec<u8> {
    //     todo!()
    // }

    // fn visit_number(&mut self, number: &ast::Number) -> Vec<u8> {
    //     todo!()
    // }
}
