mod defs;
mod sections;

use std::{
    borrow::BorrowMut,
    collections::{BTreeMap, HashMap},
    fs,
    io::Write,
    mem,
    time::SystemTime,
};

use sections::*;

use crate::emitter::data::DataRef;

use super::stdlib::windows as stdlib;
use crate::emitter::abi::windows as abi;

use self::defs::*;

use super::{
    ast,
    data::{Data, DataBuilder},
    mnemonics::*,
    structs::{Instructions, InstructionsTrait, Sliceable},
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

    let mut instructions = exe_emitter.visit_statement_list(ast);
    // dbg!(&instructions);

    let created_at = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;

    let calls = exe_emitter.calls;
    // let mut calls = BTreeMap::new();
    // calls.insert(4, "__acrt_iob_func".to_string());
    // calls.insert(10, "__stdio_common_vfprintf".to_string());
    // calls.insert(12, "ExitProcess".to_string());

    dbg!(&calls);

    let const_data = exe_emitter.data_refs;
    dbg!(&const_data);

    // let mut instructions: Instructions = vec![
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

    let text_section_data = instructions.to_bin();

    let mut data_section_data: Vec<u8> = vec![];

    for data in const_data.values() {
        data_section_data.extend(data.data.to_owned());
    }
    if data_section_data.is_empty() {
        data_section_data.push(0);
    }

    let relocation_section_data = build_relocation_section(&const_data, &instructions);

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

    let relocation_section_data = build_relocation_section(&const_data, &instructions);

    let import_directory_data =
        build_import_directory(import_directory_section.virtual_address, libs.clone());

    let external_symbols = import_directory_data.get_external_symbols(libs);
    instructions = compute_calls(
        instructions,
        text_section.virtual_address,
        calls,
        external_symbols,
    );

    let mut data_cursor = 0;
    for (line, data_ref) in const_data.iter() {
        let address = IMAGE_BASE + data_section.virtual_address as u64 + data_cursor as u64;
        println!("address: {:0x}", address);
        instructions[*line].set_op2(Operand::Imm64(address));
        data_cursor += data_ref.data.len();
    }

    let text_section_data: Vec<u8> = instructions.to_bin();

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
    mut instructions: Instructions,
    virtual_address: u32,
    calls: BTreeMap<usize, String>,
    external_symbols: HashMap<String, u32>,
) -> Instructions {
    let text_section_data = instructions.to_bin();
    for (i, (c, call)) in calls.iter().enumerate() {
        let mut address = external_symbols[call];
        // println!("{}: {:0x}", call, address);
        address -=
            virtual_address + text_section_data.len() as u32 + ((i + 1) * SIZE_OF_JMP) as u32;
        // println!("{}: {:0x}", call, address);
        instructions.push(JMP.op1(Operand::Imm32(address)));
        let call_address = text_section_data.len() as u32
            - instructions[..*c + 1].to_vec().to_bin().len() as u32
            + ((i) * SIZE_OF_JMP) as u32;
        // println!(
        //     "{:0x}: {:0x}",
        //     text_section_data.len(),
        //     instructions[..*c + 1].to_vec().to_bin().len()
        // );
        // println!("{}: {:0x}", call, call_address);

        instructions[*c].set_op1(Operand::Offset32(call_address));
    }
    instructions
}

pub struct ExeEmitter {
    literals: HashMap<ast::Ident, Data>,
    calls: BTreeMap<usize, String>,
    data_refs: BTreeMap<usize, DataRef>,
}

impl ExeEmitter {
    fn new(data_builder: &DataBuilder) -> Self {
        ExeEmitter {
            literals: data_builder.variables.clone(),
            calls: BTreeMap::new(),
            data_refs: BTreeMap::new(),
        }
    }

    fn visit_statement_list(&mut self, statement_list: &ast::StatementList) -> Instructions {
        let mut result: Instructions = vec![
            PUSH.op1(Operand::Register(register::RBP)),
            MOV.op1(Operand::Register(register::RBP))
                .op2(Operand::Register(register::RSP)),
        ];
        result.extend(statement_list.0.iter().fold(vec![], |mut r, stmt| {
            r.extend(self.visit_statement(result.len() + r.len(), stmt));
            r
        }));
        result.extend(stdlib::exit(0, &mut self.calls, result.len()));
        result
    }

    fn visit_statement(&mut self, pc: usize, statement: &ast::Statement) -> Instructions {
        // println!("{}: {:#?}", statement_n, &statement);
        match statement {
            ast::Statement::Expression(expr) => self.visit_expression(pc, expr),
            ast::Statement::Assignment(ast::Assignment(_, expr, ast::AssignmentType::Let)) => {
                match expr {
                    ast::Expression::Literal(ast::Literal::String(s)) => {
                        let mut pushes: Vec<_> = s.as_bytes().chunks(8).rev().fold(
                            vec![],
                            |mut acc: Instructions, substr| {
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
                            },
                        );
                        dbg!(pushes.len());
                        if pushes.len() % 4 != 0 {
                            pushes.push(
                                SUB.op1(Operand::Register(register::RSP))
                                    .op2(Operand::Imm32(8)),
                            );
                        }
                        pushes
                    }
                    ast::Expression::Literal(ast::Literal::Number(n)) => {
                        vec![
                            MOV.op1(Operand::Register(register::RAX))
                                .op1(Operand::Imm64(n.value as u64)),
                            PUSH.op1(Operand::Register(register::RAX)),
                        ]
                    }
                    _ => vec![],
                }
            }
            ast::Statement::Assignment(ast::Assignment(_, _, ast::AssignmentType::Const)) => vec![],
        }
    }

    fn visit_expression(&mut self, pc: usize, expr: &ast::Expression) -> Instructions {
        match expr {
            ast::Expression::Call(id, expr) => self.visit_call(pc, id, expr),

            _ => vec![],
        }
    }

    fn visit_call(&mut self, pc: usize, id: &ast::Ident, expr: &ast::Expression) -> Instructions {
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

            let mut result = vec![];
            result.extend(abi::push_args(args));
            let pc: usize = pc + result.len();
            if data.assign_type == ast::AssignmentType::Const {
                let mut offset = 1;
                if args.len() % 2 != 0 {
                    offset += 1;
                }
                self.data_refs.insert(
                    pc - offset,
                    DataRef {
                        offset: result[result.len() - offset].get_value_loc(),
                        ref_len: mem::size_of::<i64>(),
                        data: match &data.lit {
                            ast::Literal::String(s) => [s.as_bytes().to_vec(), vec![0]].concat(),
                            ast::Literal::Number(_) => b"%d\0".to_vec(),
                        },
                    },
                );
            }

            let print_call = match data.lit {
                ast::Literal::String(_) => stdlib::print(&mut self.calls, pc),
                ast::Literal::Number(n) => {
                    stdlib::printd(&mut self.calls, &mut self.data_refs, n.value, pc)
                }
            };

            result.extend(print_call);
            result.extend(abi::pop_args(args.len()));
            return result;
        }

        panic!("no such function {}", id.value)
    }
}
