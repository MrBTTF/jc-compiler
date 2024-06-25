mod defs;
pub mod sections;

use std::{collections::BTreeMap, env, fs, io::Write, mem, time::SystemTime};

use sections::*;

use self::defs::*;

use super::{ast, code_context::{CodeContext, Sliceable}, data::{Data, DataBuilder}, Emitter, mnemonics::*};

pub fn build(output_path: &str, emitter: &Emitter, variables: &BTreeMap<ast::Ident, Data>) {
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

    let mut code_context = emitter.get_code_context();
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
