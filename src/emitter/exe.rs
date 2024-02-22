mod defs;
mod sections;

use std::{
    borrow::{Borrow, BorrowMut},
    collections::{BTreeMap, HashMap},
    fs,
    io::Write,
    mem,
    rc::Rc,
    time::SystemTime,
};

use sections::*;

use crate::emitter::structs::Instruction;

use self::defs::*;

use super::{
    amd64::*,
    ast,
    data::{Data, DataBuilder},
    structs::{Instructions, InstructionsTrait},
};

pub fn build_exe(_ast: &ast::StatementList) {
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

    // let mut data_builder = DataBuilder::default();
    // data_builder.visit_statement_list(ast);

    // let mut exe_emitter = ExeEmitter::new(&data_builder);

    // let instructions = exe_emitter.visit_statement_list(ast);
    // // dbg!(&instructions);
    // let text_header = instructions.to_bin();

    let created_at = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;


    let mut calls = BTreeMap::new();
    calls.insert(4, "__acrt_iob_func".to_string());
    calls.insert(10, "__stdio_common_vfprintf".to_string());
    calls.insert(12, "ExitProcess".to_string());

    let mut instructions: Instructions = vec![
        // text_section.push(  Push::new(Register::Bp));
        // text_section.push(  Mov64rr::new(Register::Bp, Register::Sp));
        Push::new(Register::Bp),
        Mov64rr::new(Register::Bp, Register::Sp),
        Sub64::new(Register::Sp, 0x20),
        Mov32::new(Register::Cx, 0x1),
        Call::new(0x0),
        Mov64rr::new(Register::Bx, Register::Ax),
        Mov32::new(Register::Cx, 0x0),
        Mov64rr::new(Register::Dx, Register::Bx),
        Mov64Ext::new(RegisterExt::R8, 0x140003000),
        Mov32Ext::new(RegisterExt::R9, 0x0),
        Call::new(0x0),
        Xor64rr::new(Register::Ax, Register::Ax),
        Call::new(0x0),
    ];

  
    let text_section_bin = instructions.to_bin();

    let data_section: Vec<u8> = vec![
        0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x0D, 0x0A, 0x00,
    ];

    let mut section_layout = SectionLayout::new(vec![
        Section::new(
            ".text".to_string(),
            SectionData::Data(text_section_bin.clone()),
            SECTION_CHARACTERISTICS_TEXT
                | SECTION_CHARACTERISTICS_EXEC
                | SECTION_CHARACTERISTICS_READ,
        ),
        Section::new(
            ".rdata".to_string(),
            SectionData::ImportData(build_import_directory, libs),
            SECTION_CHARACTERISTICS_DATA | SECTION_CHARACTERISTICS_READ,
        ),
        Section::new(
            ".data".to_string(),
            SectionData::Data(data_section.clone()),
            SECTION_CHARACTERISTICS_DATA
                | SECTION_CHARACTERISTICS_READ
                | SECTION_CHARACTERISTICS_WRITE,
        ),
        Section::new(
            ".reloc".to_string(),
            SectionData::Data(build_relocation_section().clone()),
            SECTION_CHARACTERISTICS_DATA
                | SECTION_CHARACTERISTICS_READ
                | SECTION_CHARACTERISTICS_DISCARDABLE,
        ),
    ]);

    let mut text_section = section_layout.get_section(".text");
    let import_directory_section = section_layout.get_section(".rdata");
    let data_section = section_layout.get_section(".data");
    let relocation_section = section_layout.get_section(".reloc");

    for (i, (c, call)) in calls.iter().enumerate() {
        let mut address = section_layout.external_symbols[call];
        println!("{}: {:0x}", call, address);
        address -= text_section.virtual_address
            + text_section_bin.len() as u32
            + ((i + 1) * mem::size_of::<Jmp>()) as u32;
        println!("{}: {:0x}", call, address);
        instructions.push(Jmp::new(address));
        let call_address = text_section_bin.len() as u32
            - instructions[..*c + 1].to_vec().to_bin().len() as u32
            + ((i) * mem::size_of::<Jmp>()) as u32;

        let call_instruction = instructions[*c].borrow_mut();
        *call_instruction = Call::new(call_address as u16);
    }
    text_section.set_data(SectionData::Data(instructions.to_bin()));
    section_layout.set_section(".text", text_section.clone());

    //TODO: Debug relocations

    let dos_header = build_dos_header();
    let nt_header = build_nt_header(created_at, section_layout);

    let mut headers = [
        dos_header,
        nt_header,
        text_section.get_header(),
        import_directory_section.get_header(),
        data_section.get_header(),
        relocation_section.get_header(),
    ]
    .concat();
    let diff = 2 * FILE_ALIGNMENT as usize - headers.len();
    headers.extend(std::iter::repeat(0).take(diff));
    // println!("{:#?}", import_directory);

    let mut file = fs::File::create("bin/hello.exe").unwrap();
    file.write_all(&headers).unwrap();
    file.write_all(&text_section.get_data_aligned()).unwrap();
    file.write_all(&import_directory_section.get_data_aligned())
        .unwrap();
    file.write_all(&data_section.get_data_aligned()).unwrap();
    file.write_all(&relocation_section.get_data_aligned())
        .unwrap();
}

// pub struct ExeEmitter {
//     literals: HashMap<ast::Ident, Data>,
// }

// impl ExeEmitter {
//     fn new(data_builder: &DataBuilder) -> Self {
//         ExeEmitter {
//             literals: data_builder.variables.clone(),
//         }
//     }

//     fn visit_statement_list(&mut self, statement_list: &ast::StatementList) -> Instructions {
//         let mut result: Instructions = vec![];
//         result.push(Mov64rr::new(Register::Bp, Register::Sp));
//         result.extend(statement_list.0.iter().fold(vec![], |mut result, stmt| {
//             result.extend(self.visit_statement(stmt));
//             result
//         }));
//         result.extend(stdlib::exit(0));
//         result
//     }

//     fn visit_statement(&mut self, statement: &ast::Statement) -> Instructions {
//         match statement {
//             ast::Statement::Expression(expr) => self.visit_expression(expr),
//             ast::Statement::Assignment(ast::Assignment(_, expr, ast::AssignmentType::Let)) => {
//                 match expr {
//                     ast::Expression::Literal(ast::Literal::String(s)) => {
//                         let pushes: Vec<_> = s.as_bytes().chunks(8).rev().fold(
//                             vec![],
//                             |mut acc: Instructions, substr| {
//                                 let mut value: u64 = 0;
//                                 for (i, c) in substr.iter().enumerate() {
//                                     value += (*c as u64) << (8 * i)
//                                 }
//                                 acc.push(Mov64::new(Register::Ax, value as i64));
//                                 acc.push(Push::new(Register::Ax));
//                                 acc
//                             },
//                         );
//                         pushes
//                     }
//                     ast::Expression::Literal(ast::Literal::Number(n)) => {
//                         vec![Mov64::new(Register::Ax, n.value), Push::new(Register::Ax)]
//                     }
//                     _ => vec![],
//                 }
//             }
//             ast::Statement::Assignment(ast::Assignment(_, _, ast::AssignmentType::Const)) => vec![],
//         }
//     }

//     fn visit_expression(&mut self, expr: &ast::Expression) -> Instructions {
//         match expr {
//             ast::Expression::Call(id, expr) => self.visit_call(id, expr),

//             _ => vec![],
//         }
//     }

//     fn visit_call(&mut self, id: &ast::Ident, expr: &ast::Expression) -> Instructions {
//         let data = match expr {
//             ast::Expression::Ident(id) => self
//                 .literals
//                 .get(id)
//                 .unwrap_or_else(|| panic!("undefined variable: {}", id.value)),
//             _ => todo!(),
//         };

//         if id.value == "print" {
//             let args = &[match data.assign_type {
//                 ast::AssignmentType::Let => abi::Arg::Stack(data.data_loc() as i64),
//                 ast::AssignmentType::Const => abi::Arg::Data(data.data_loc() as i64),
//             }];
//             let print_call = match data.lit {
//                 ast::Literal::String(_) => stdlib::print(data.lit.len()),
//                 ast::Literal::Number(_) => stdlib::printd(),
//             };
//             let mut result = vec![];
//             result.extend(abi::push_args(args));
//             result.extend(print_call);
//             result.extend(abi::pop_args(args.len()));
//             return result;
//         }

//         panic!("no such function {}", id.value)
//     }
// }
