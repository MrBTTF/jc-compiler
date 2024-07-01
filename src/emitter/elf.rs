pub mod defs;
pub mod sections;

use std::{collections::BTreeMap, fmt::Result, fs, io::Write, mem};

use self::{
    super::{ast, data::*},
    sections::*,
};

use super::{
    code_context::{self, CodeContext, Sliceable},
    Emitter, Ident,
};

pub fn build(output_path: &str, emitter: Emitter, variables: &BTreeMap<ast::Ident, Data>) {
    let data_section_data = build_data_section(&variables);

    let code_context = emitter.get_code_context();
    // dbg!(&code_context);

    let text_section_data = code_context.to_bin();

    let section_names: &[&str] = &[
        ".text",
        ".data",
        ".shstrtab",
        ".symtab",
        ".strtab",
        ".rela.text",
    ];
    let shstrtab_section_data: Vec<u8> = build_shstrtab_section(section_names);
    let symbols = build_symbols("local/bin/hello.o", &code_context, &variables);
    let symstr = SymStr::new(symbols.as_slice());
    let symtab_section_data = symstr.get_symtab();
    let strtab_section_data = symstr.get_strtab();
    let relocation_section_data: Vec<u8> = build_rel_text_section(&symbols);

    let section_headers = &[
        SectionHeader::new(
            text_section_data.len(),
            defs::SEGMENT_TYPE_PROGBITS,
            defs::SEGMENT_FLAGS_ALLOC | defs::SEGMENT_FLAGS_EXECINSTR,
            16,
            0,
            0,
            0,
        ),
        SectionHeader::new(
            data_section_data.len(),
            defs::SEGMENT_TYPE_PROGBITS,
            defs::SEGMENT_FLAGS_WRITE | defs::SEGMENT_FLAGS_ALLOC,
            4,
            0,
            0,
            0,
        ),
        SectionHeader::new(
            shstrtab_section_data.len(),
            defs::SEGMENT_TYPE_STRTAB,
            defs::SEGMENT_FLAGS_NONE,
            1,
            0,
            0,
            0,
        ),
        SectionHeader::new(
            symtab_section_data.len(),
            defs::SEGMENT_TYPE_SYMTAB,
            defs::SEGMENT_FLAGS_NONE,
            8,
            mem::size_of::<sections::SymbolTable>() as u64,
            symbols.len() as u32, // TODO: compute
            5,
        ),
        SectionHeader::new(
            strtab_section_data.len(),
            defs::SEGMENT_TYPE_STRTAB,
            defs::SEGMENT_FLAGS_NONE,
            1,
            0,
            0,
            0,
        ),
        SectionHeader::new(
            relocation_section_data.len(),
            defs::SEGMENT_TYPE_RELA,
            defs::SEGMENT_FLAGS_NONE,
            8,
            mem::size_of::<sections::RelocationTable>() as u64,
            1,
            4,
        ),
    ];

    // let program_headers = &build_program_headers(text_section.len(), data_section.len());
    let header = &build_header(section_headers, 3);

    let header_data = header.as_slice();

    let mut file = fs::File::create(output_path).unwrap();
    file.write_all(header_data).unwrap();
    // file.write_all(program_headers).unwrap();

    file.write_all(&build_section_headers(section_headers, section_names))
        .unwrap();

    file.write_all(&text_section_data).unwrap();
    file.write_all(&data_section_data).unwrap();
    file.write_all(&shstrtab_section_data).unwrap();
    file.write_all(&symtab_section_data).unwrap();
    file.write_all(&strtab_section_data).unwrap();
    file.write_all(&relocation_section_data).unwrap();
}

fn build_symbols(
    filename: &str,
    code_context: &CodeContext,
    variables: &BTreeMap<ast::Ident, Data>,
) -> Vec<Symbol> {
    let file = Symbol {
        name: filename.to_string(),
        offset: 0,
        _type: defs::STT_FILE,
        bind: defs::STB_LOCAL,
        ..Default::default()
    };
    let text = Symbol {
        offset: 0,
        section: Some(1),
        _type: defs::STT_SECTION,
        bind: defs::STB_LOCAL,
        ..Default::default()
    };
    let data = Symbol {
        offset: 0,
        section: Some(2),
        _type: defs::STT_SECTION,
        bind: defs::STB_LOCAL,
        ..Default::default()
    };
    let mut local_vars = vec![];
    for (pc, d) in code_context.get_const_data() {
        let offset = variables
            .get(&Ident {
                value: d.symbol.to_string(),
            })
            .unwrap()
            .data_loc;
        let data_loc = code_context.get_offset(pc) + d.offset;
        let symbol = Symbol {
            name: d.symbol.to_string(),
            offset,
            data_loc: data_loc as u64,
            section: Some(2),
            _type: defs::STT_OBJECT,
            bind: defs::STB_LOCAL,
            ..Default::default()
        };
        local_vars.push(symbol);
    }

    let start = Symbol {
        name: "_start".to_string(),
        offset: 0,
        section: Some(1),
        _type: defs::STT_NOTYPE,
        bind: defs::STB_GLOBAL,
        ..Default::default()
    };

    let mut result = vec![file, text, data];
    result.extend(local_vars);
    result.push(start);

    result
}

fn align(mut v: Vec<u8>, alignment: usize) -> Vec<u8> {
    let reminder = v.len() % alignment;
    let new_len = v.len() + alignment - reminder;
    v.resize(new_len, 0);
    v
}
