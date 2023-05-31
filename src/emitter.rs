use std::{fs, io::Write};

use self::elf::structs::*;

mod amd64;
pub mod elf;

pub fn build_elf() {
    let data_header = &build_data_section();
    let text_header = &build_text_section();
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
