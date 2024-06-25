pub mod defs;
pub mod sections;

use std::{
    collections::BTreeMap,
    fs::{self},
    io::Write,
};

use self::{
    super::{ast, data::*},
    sections::*,
};

use super::{code_context::Sliceable, Emitter};

pub fn build(output_path: &str, emitter: Emitter, variables: &BTreeMap<ast::Ident, Data>) {
    let data_section = &build_data_section(variables.clone());

    let code_context = emitter.get_code_context();
    // dbg!(&code_context);

    let text_section = code_context.to_bin();

    let shstrtab_header = &build_shstrtab_section();
    let program_headers = &build_program_headers(text_section.len(), data_section.len());
    let header = &build_header(
        data_section.len(),
        program_headers.len() + data_section.len() + text_section.len() + shstrtab_header.len(),
    );

    let header_data = header.as_slice();

    let mut file = fs::File::create(output_path).unwrap();
    file.write_all(header_data).unwrap();
    file.write_all(program_headers).unwrap();

    file.write_all(data_section).unwrap();
    file.write_all(&text_section).unwrap();
    file.write_all(shstrtab_header).unwrap();

    file.write_all(&build_section_headers(
        text_section.len(),
        data_section.len(),
        shstrtab_header.len(),
    ))
    .unwrap();
}
