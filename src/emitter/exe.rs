mod defs;
pub mod sections;

use std::{collections::BTreeMap, env, fs, io::Write, mem, time::SystemTime};

use sections::*;

use super::{stdlib::windows as stdlib};
use crate::emitter::abi::windows as abi;
use crate::emitter::ast::{DeclarationType, Expression, Literal};
use crate::emitter::elf::sections::DWord;

use self::defs::*;

use super::{
    ast,
    data::{Data, DataBuilder},
    mnemonics::*,
    code_context::{CodeContext, Sliceable},
};

pub fn build_exe(ast: &ast::StatementList, output_path: &str) {
    let mut libs = BTreeMap::new();
    libs.insert(
        "KERNEL32.dll\0".as_bytes().to_vec(),
        vec![HintEntry {
            hint: 0x167_u16,
            name: "ExitProcess\0".as_bytes().to_vec(),
        }],
    );

    libs.insert(
        "api-ms-win-crt-stdio-l1-1-0.dll\0".as_bytes().to_vec(),
        vec![
            HintEntry {
                hint: 0_u16,
                name: "__acrt_iob_func\0".as_bytes().to_vec(),
            },
            HintEntry {
                hint: 0x3_u16,
                name: "__stdio_common_vfprintf\0".as_bytes().to_vec(),
            },
        ],
    );

    let mut data_builder = DataBuilder::default();
    data_builder.visit_statement_list(ast);
    dbg!(&data_builder.variables);

    let mut exe_emitter = ExeEmitter::new(&data_builder);

    exe_emitter.visit_statement_list(ast);
    let mut code_context = exe_emitter.get_code_context();
    let mut debug_code_file =
        fs::File::create(env::current_dir().unwrap().join("local/debug/code.txt")).unwrap();
    debug_code_file
        .write_all(format!("{:#?}", &code_context).as_bytes())
        .unwrap();

    let created_at = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;

    let calls = code_context.get_calls();
    // let mut calls = BTreeMap::new();
    // calls.insert(4, "__acrt_iob_func".to_string());
    // calls.insert(10, "__stdio_common_vfprintf".to_string());
    // calls.insert(12, "ExitProcess".to_string());

    dbg!(&calls);

    let const_data = code_context.get_const_data();
    // dbg!(&const_data);

    let mut data_section_data: Vec<u8> = vec![];

    for data in const_data.values() {
        data_section_data.extend(data.data.to_owned());
    }
    if data_section_data.is_empty() {
        data_section_data.push(0);
    }

    let relocation_section_data = build_relocation_section(&const_data, &code_context);

    let section_layout = SectionLayout::new(vec![
        Section::new(
            ".text".to_string(),
            code_context.get_code_size_with_calls() as u32,
            SECTION_CHARACTERISTICS_TEXT
                | SECTION_CHARACTERISTICS_EXEC
                | SECTION_CHARACTERISTICS_READ,
        ),
        Section::new(
            ".rdata".to_string(),
            build_import_directory(0, libs.clone()).as_vec().len() as u32,
            SECTION_CHARACTERISTICS_DATA | SECTION_CHARACTERISTICS_READ,
        ),
        Section::new(
            ".data".to_string(),
            data_section_data.len() as u32,
            SECTION_CHARACTERISTICS_DATA
                | SECTION_CHARACTERISTICS_READ
                | SECTION_CHARACTERISTICS_WRITE,
        ),
        Section::new(
            ".reloc".to_string(),
            relocation_section_data.len() as u32,
            SECTION_CHARACTERISTICS_DATA
                | SECTION_CHARACTERISTICS_READ
                | SECTION_CHARACTERISTICS_DISCARDABLE,
        ),
    ]);

    let text_section = section_layout.get_section(".text");
    let import_directory_section = section_layout.get_section(".rdata");
    let data_section = section_layout.get_section(".data");
    let relocation_section = section_layout.get_section(".reloc");

    let relocation_section_data = build_relocation_section(&const_data, &code_context);

    let import_directory_data =
        build_import_directory(import_directory_section.virtual_address, libs.clone());

    let external_symbols = import_directory_data.get_external_symbols(libs);
    code_context.compute_calls(text_section.virtual_address, external_symbols);
    code_context.compute_data(data_section.virtual_address, const_data);

    let text_section_data: Vec<u8> = code_context.to_bin();

    let dos_header = build_dos_header();
    let nt_header = build_nt_header(created_at, section_layout);

    let headers = [
        dos_header,
        nt_header,
        text_section.get_header(),
        import_directory_section.get_header(),
        data_section.get_header(),
        relocation_section.get_header(),
    ]
        .concat();

    let mut file = fs::File::create(output_path).unwrap();
    write_all_aligned(&mut file, &headers).unwrap();
    write_all_aligned(&mut file, &text_section_data).unwrap();
    write_all_aligned(&mut file, &import_directory_data.as_vec()).unwrap();
    write_all_aligned(&mut file, &data_section_data).unwrap();
    write_all_aligned(&mut file, &relocation_section_data).unwrap();
}

fn write_all_aligned(file: &mut fs::File, buf: &[u8]) -> Result<(), std::io::Error> {
    let buf = get_data_aligned(buf.to_vec());
    file.write_all(&buf)
}

pub struct ExeEmitter {
    code_context: CodeContext,
    literals: BTreeMap<ast::Ident, Data>,
    data_ordered: Vec<ast::Ident>,
}

impl ExeEmitter {
    fn new(data_builder: &DataBuilder) -> Self {
        ExeEmitter {
            code_context: CodeContext::new(),
            literals: data_builder.variables.clone(),
            data_ordered: data_builder.data_ordered.clone(),
        }
    }

    pub fn get_code_context(&self) -> CodeContext {
        self.code_context.clone()
    }

    fn visit_statement_list(&mut self, statement_list: &ast::StatementList) {
        self.code_context.add_slice(&[
            PUSH.op1(register::RBP),
            MOV.op1(register::RBP).op2(register::RSP),
        ]);
        self.code_context.add_slice(&self.allocate_stack());
        statement_list.0.iter().for_each(|stmt| {
            self.visit_statement(stmt);
        });
        stdlib::exit(&mut self.code_context, 0);
    }

    fn visit_statement(&mut self, statement: &ast::Statement) {
        // println!("{}: {:#?}", statement_n, &statement);
        match statement {
            ast::Statement::Expression(expr) => {
                self.visit_expression(expr);
            }
            ast::Statement::Declaration(_) => (),
            ast::Statement::Assignment(assign) => {
                self.visit_assignment(assign);
            }
        };
    }

    fn visit_expression(&mut self, expr: &ast::Expression) {
        match expr {
            ast::Expression::Call(id, expr) => {
                self.visit_call(id, expr);
            }
            ast::Expression::Loop(l) => self.visit_loop(l),
            _ => (),
        }
    }

    fn visit_assignment(&mut self, assign: &ast::Assignment) {
        let ast::Assignment(id, expr) = assign;

        match expr {
            Expression::Literal(lit) => {
                let values = match lit {
                    Literal::String(s) => str_to_u64(s),
                    Literal::Number(n) => vec![n.value as u64],
                };

                let data = self
                    .literals
                    .get_mut(id)
                    .unwrap_or_else(|| panic!("undefined variable: {}", id.value));
                if data.decl_type == DeclarationType::Const {
                    panic!("Cannot assign to const data: {data:#?}");
                }
                data.lit = lit.clone();

                let assign_at_stack_location = values.iter().enumerate().fold(vec![], |mut acc: Vec<Mnemonic>, (i, v)| {
                    acc.push(MOV.op1(register::RAX).op2(*v));
                    acc.push(PUSH.op1(register::RAX));
                    acc
                });

                self.code_context.add_slice(&[
                    PUSH.op1(register::RAX),
                    PUSH.op1(register::RBX),
                    MOV.op1(register::RBX).op2(register::RSP),
                    MOV.op1(register::RSP).op2(register::RBP),
                ]);
                self.code_context.add_slice(assign_at_stack_location.as_slice());
                self.code_context.add_slice(&[
                    MOV.op1(register::RSP).op2(register::RBX),
                    POP.op1(register::RBX),
                    POP.op1(register::RAX)
                ]);
            }
            Expression::Ident(_) => todo!(),
            Expression::Call(_, _) => todo!(),
            Expression::Loop(_) => todo!(),
        };
    }

    fn visit_call(&mut self, id: &ast::Ident, expr: &ast::Expression) {
        let data = match expr {
            ast::Expression::Ident(id) => self
                .literals
                .get(id)
                .unwrap_or_else(|| panic!("undefined variable: {}", id.value)),
            _ => todo!(),
        };

        if id.value == "print" {
            match data.lit {
                ast::Literal::String(_) => {
                    let args = &[data.clone()];
                    abi::push_args(&mut self.code_context, args);

                    stdlib::print(&mut self.code_context);

                    abi::pop_args(&mut self.code_context, args.len());
                }
                ast::Literal::Number(n) => {
                    let format = self
                        .literals
                        .get(&ast::Ident {
                            value: "__printf_d_arg".to_string(),
                        })
                        .unwrap_or_else(|| panic!("undefined variable: {}", id.value));

                    let args = &[format.clone(), data.clone()];

                    abi::push_args(&mut self.code_context, args);

                    stdlib::printd(&mut self.code_context);

                    abi::pop_args(&mut self.code_context, args.len());
                }
            };

            return;
        }

        panic!("no such function {}", id.value)
    }

    fn visit_loop(&mut self, l: &ast::Loop) {
        let counter = self
            .literals
            .get(&l.var)
            .unwrap_or_else(|| panic!("undefined variable: {}", l.var.value)).clone();

        let offset = self.code_context.get_code_size();

        l.body.iter().for_each(|stmt| {
            self.visit_statement(stmt);
        });
        self.code_context.add(MOV.op1(register::RCX).op2(register::RBP));
        self.code_context.add(SUB.op1(register::RCX).op2(counter.data_loc() as u32));
        self.code_context.add(INC.op1(register::RCX).disp(Operand::Offset32(0)));
        self.code_context.add(CMP.op1(register::RCX).op2(l.end as u32).disp(Operand::Offset32(0)));

        let jump = JL.op1(Operand::Offset32(-(0 as i32))).as_vec().len() + self.code_context.get_code_size() - offset;
        dbg!(jump);
        self.code_context.add(JL.op1(Operand::Offset32(-(jump as i32))));
    }


    fn allocate_stack(&self) -> Vec<Mnemonic> {
        let mut result = vec![];
        for id in self.data_ordered.iter() {
            let data = self.literals.get(id).unwrap();
            if data.decl_type == ast::DeclarationType::Const {
                continue;
            }
            result.extend(match data.decl_type {
                ast::DeclarationType::Let => match &data.lit {
                    ast::Literal::String(s) => push_string_on_stack(s),
                    ast::Literal::Number(n) => vec![
                        MOV.op1(register::RAX).op2(n.value as u64),
                        PUSH.op1(register::RAX),
                    ],
                },
                ast::DeclarationType::Const => vec![],
            });
        }

        if result.len() % 4 != 0 {
            result.push(SUB.op1(register::RSP).op2(8_u32));
        }
        result
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

fn push_string_on_stack(s: &str) -> Vec<Mnemonic> {
    str_to_u64(s).iter().fold(vec![], |mut acc: Vec<Mnemonic>, value| {
        acc.push(MOV.op1(register::RAX).op2(*value));
        acc.push(PUSH.op1(register::RAX));
        acc
    })
}
