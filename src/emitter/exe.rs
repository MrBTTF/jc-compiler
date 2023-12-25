mod defs;
mod sections;

use std::{collections::HashMap, fs, io::Write, mem, time::SystemTime};

use sections::*;

use self::defs::*;

use super::{
    ast,
    data::{Data, DataBuilder},
};

pub fn build_exe(ast: &ast::StatementList) {
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

    let size_of_dos_header = mem::size_of::<DOSHeader>() as u32;
    let size_of_optional_header = mem::size_of::<OptionalHeader>() as u32;
    let number_of_sections = 4;
    let size_of_section_header = number_of_sections as u32 * mem::size_of::<SectionHeader>() as u32;

    let size_of_headers = size_of_dos_header + size_of_optional_header + size_of_section_header;
    let size_of_headers = round_to_multiple(size_of_headers, FILE_ALIGNMENT);

    let dos_header = build_dos_header();
    let nt_header = build_nt_header(
        created_at,
        number_of_sections,
        size_of_optional_header,
        size_of_headers,
        0x200,
        0x600,
        0x0,
    );

    let text_section: Vec<u8> = vec![
        0x55, 0x48, 0x89, 0xE5, 0x48, 0x83, 0xEC, 0x20, 0xB9, 0x01, 0x00, 0x00, 0x00, 0xE8, 0x3E,
        0x00, 0x00, 0x00, 0x48, 0x89, 0xC3, 0xB9, 0x00, 0x00, 0x00, 0x00, 0x48, 0x89, 0xDA, 0x49,
        0xB8, 0x00, 0x30, 0x00, 0x40, 0x01, 0x00, 0x00, 0x00, 0x41, 0xB9, 0x00, 0x00, 0x00, 0x00,
        0xE8, 0x2E, 0x00, 0x00, 0x00, 0x48, 0x31, 0xC0, 0xE8, 0x06, 0x00, 0x00, 0x00, 0xCC, 0xCC,
        0xCC, 0xCC, 0xCC, 0xCC, 0xFF, 0x25, 0x1E, 0x10, 0x0, 0x0, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC,
        0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xFF, 0x25, 0x1E, 0x10, 0x0, 0x0, 0xCC, 0xCC, 0xCC, 0xCC,
        0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xFF, 0x25, 0x16, 0x10, 0x0, 0x0, 0xCC, 0xCC, 0xCC,
        0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC,
    ];

    let data_section: Vec<u8> = vec![
        0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x0D, 0x0A, 0x00,
    ];

    let pe_layout = PELayout::new(vec![
        Section::new(
            ".text".to_string(),
            SectionData::Data(text_section.clone()),
            SECTION_CHARACTERISTICS_TEXT
                | SECTION_CHARACTERISTICS_EXEC
                | SECTION_CHARACTERISTICS_READ,
        ),
        Section::new(
            ".rdata".to_string(),
            SectionData::DataCallback(build_import_directory),
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

    let text_section = pe_layout.get_section(".text");
    let import_directory_section = pe_layout.get_section(".rdata");
    let data_section = pe_layout.get_section(".data");
    let relocation_section = pe_layout.get_section(".reloc");

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
