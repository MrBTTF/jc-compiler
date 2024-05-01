mod defs;
mod sections;

use std::{
    borrow::BorrowMut,
    collections::{BTreeMap, HashMap},
    fs,
    io::Write,
    mem, result,
    time::SystemTime,
};

use sections::*;

use crate::emitter::data::DataRef;

use super::{ast::AssignmentType, stdlib::windows as stdlib};
use crate::emitter::abi::windows as abi;

use self::defs::*;

use super::{
    ast,
    data::{Data, DataBuilder},
    mnemonics::*,
    structs::{CodeContext, Sliceable},
};

const SIZE_OF_JMP: usize = mem::size_of::<u8>() * 2 + mem::size_of::<u32>();

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
    // dbg!(&code_context);

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
    dbg!(&const_data);

    // let mut code_context: Instructions = vec![
    //     Push::new(Register::Bp),
    //     Mov64rr::new(Register::Bp, Register::Sp),
    //     Sub64::new(Register::Sp, 0x20),
    //     Mov32::new(Register::Cx, 0x1),
    //     Call::new(0x0),
    //     Mov64rr::new(Register::Bx, Register::Ax),
    //     Mov32::new(Register::Cx, 0x0),
    //     Mov64rr::new(Register::Dx, Register::Bx),
    //     Mov64Ext::new(RegisterExt::R8, 0x140003000),
    //     Mov32Ext::new(RegisterExt::R9, 0x0),
    //     Call::new(0x0),
    //     Xor64rr::new(Register::Ax, Register::Ax),
    //     Call::new(0x0),
    // ];

    let text_section_data = code_context.to_bin();

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
            (text_section_data.len() + SIZE_OF_JMP * calls.len()) as u32,
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
    code_context = compute_calls(
        code_context.clone(),
        text_section.virtual_address,
        &calls,
        external_symbols,
    );

    let mut data_cursor = 0;
    for (line, data_ref) in const_data.iter() {
        let address = IMAGE_BASE + data_section.virtual_address as u64 + data_cursor as u64;
        println!("address: {:0x}", address);
        code_context.get_mut(*line).set_op2(Operand::Imm64(address));
        data_cursor += data_ref.data.len();
    }

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

fn compute_calls(
    mut code_context: CodeContext,
    virtual_address: u32,
    calls: &BTreeMap<String, Vec<usize>>,
    external_symbols: HashMap<String, u32>,
) -> CodeContext {
    let text_section_data = code_context.to_bin();
    for (i, (call, locs)) in calls.iter().enumerate() {
        for c in locs.iter() {
            assert!(
                code_context.get(*c).get_name() == "CALL",
                "line: {}\n{}",
                *c,
                code_context.get(*c)
            );
            let call_address = text_section_data.len() as u32
                - code_context.get_offset(*c + 1) as u32
                + ((i) * SIZE_OF_JMP) as u32;
            // println!(
            //     "{:0x}: {:0x}",
            //     text_section_data.len(),
            //     code_context[..*c + 1].to_vec().to_bin().len()
            // );
            // println!("{}: {:0x}", call, call_address);

            code_context
                .get_mut(*c)
                .set_op1(Operand::Offset32(call_address));
        }

        let mut address = external_symbols[call];
        // println!("{}: {:0x}", call, address);
        address -=
            virtual_address + text_section_data.len() as u32 + ((i + 1) * SIZE_OF_JMP) as u32;
        // println!("{}: {:0x}", call, address);
        code_context.add(JMP.op1(Operand::Imm32(address)));
    }
    code_context
}

pub struct ExeEmitter {
    code_context: CodeContext,
    literals: BTreeMap<ast::Ident, Data>,
}

impl ExeEmitter {
    fn new(data_builder: &DataBuilder) -> Self {
        ExeEmitter {
            code_context: CodeContext::new(),
            literals: data_builder.variables.clone(),
        }
    }

    pub fn get_code_context(&self) -> CodeContext {
        self.code_context.clone()
    }

    fn visit_statement_list(&mut self, statement_list: &ast::StatementList) {
        self.code_context.add_slice(&[
            PUSH.op1(Operand::Register(register::RBP)),
            MOV.op1(Operand::Register(register::RBP))
                .op2(Operand::Register(register::RSP)),
        ]);
        self.code_context.add_slice(&allocate_stack(&self.literals));
        statement_list.0.iter().for_each(|mut stmt| {
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
            ast::Statement::Assignment(_) => (),
        };
    }

    fn visit_expression(&mut self, expr: &ast::Expression) {
        match expr {
            ast::Expression::Call(id, expr) => {
                self.visit_call(id, expr);
            }
            _ => (),
        }
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
            let args = &[data.clone()];

            abi::push_args(&mut self.code_context, args);

            match data.lit {
                ast::Literal::String(_) => {
                    stdlib::print(&mut self.code_context);
                }
                ast::Literal::Number(n) => {
                    stdlib::printd(&mut self.code_context, n.value);
                }
            };

            abi::pop_args(&mut self.code_context, args.len());
            return;
        }

        panic!("no such function {}", id.value)
    }
}

fn allocate_stack(literals: &BTreeMap<ast::Ident, Data>) -> Vec<Mnemonic> {
    let mut result = vec![];
    for (_, data) in literals.iter() {
        if data.assign_type == AssignmentType::Const {
            continue;
        }
        result.extend(match data.assign_type {
            AssignmentType::Let => match &data.lit {
                ast::Literal::String(s) => push_string_on_stack(s),
                ast::Literal::Number(n) => vec![
                    MOV.op1(Operand::Register(register::RAX))
                        .op1(Operand::Imm64(n.value as u64)),
                    PUSH.op1(Operand::Register(register::RAX)),
                ],
            },
            AssignmentType::Const => vec![],
        });
    }

    if result.len() % 4 != 0 {
        result.push(
            SUB.op1(Operand::Register(register::RSP))
                .op2(Operand::Imm32(8)),
        );
    }
    result
}

fn push_string_on_stack(s: &str) -> Vec<Mnemonic> {
    s.as_bytes()
        .chunks(8)
        .rev()
        .fold(vec![], |mut acc: Vec<Mnemonic>, substr| {
            let mut value: u64 = 0;
            for (i, c) in substr.iter().enumerate() {
                value += (*c as u64) << (8 * i)
            }
            acc.push(
                MOV.op1(Operand::Register(register::RAX))
                    .op2(Operand::Imm64(value)),
            );
            acc.push(PUSH.op1(Operand::Register(register::RAX)));
            acc
        })
}
