pub mod defs;
pub mod sections;

use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::Result,
    fs,
    io::Write,
    mem,
    path::PathBuf,
    process::Command,
};

use defs::{SHN_ABS, SHN_UNDEF};
use elf::relocation;

use crate::emitter::symbols::DataSymbol;

use self::{
    super::{ast, data::*},
    sections::*,
};

use super::{
    symbols::{self},
    text::{CodeContext, Sliceable},
};

pub fn build(
    output_path: PathBuf,
    code_context: &CodeContext,
    symbol_data: &HashMap<String, Data>,
    symbols: &[symbols::Symbol],
    relocations: &[symbols::Relocation],
) {
    let object_file = output_path.with_extension("o");

    let data_section_data = build_data_section(&symbol_data);

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
    let (symbols, relocations, last_local_idx) =
        build_symbols(object_file.to_str().unwrap(), relocations, &symbols);
    let symstr = SymStr::new(symbols.as_slice());
    let symtab_section_data = symstr.get_symtab();
    let strtab_section_data = symstr.get_strtab();
    let relocation_section_data: Vec<u8> = build_rel_text_section(relocations.as_slice());

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
            last_local_idx as u32 + 1,
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

    let header = &build_header(section_headers, 3);

    let header_data = header.as_slice();

    let mut file = fs::File::create(&object_file).unwrap();
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

    let child = Command::new("ld")
        .args(&[
            "-lc",
            "-o",
            output_path.to_str().unwrap(),
            object_file.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    if !child.status.success() {
        panic!("{}", String::from_utf8(child.stderr).unwrap());
    }
}

fn build_symbols(
    filename: &str,
    relocations: &[symbols::Relocation],
    symbols: &[symbols::Symbol],
) -> (Vec<Symbol>, Vec<Relocation>, usize) {
    let file = Symbol {
        name: filename.to_string(),
        offset: 0,
        _type: defs::STT_FILE,
        bind: defs::STB_LOCAL,
        section: SHN_ABS,
        ..Default::default()
    };
    let text = Symbol {
        offset: 0,
        section: 1,
        _type: defs::STT_SECTION,
        bind: defs::STB_LOCAL,
        ..Default::default()
    };
    let data = Symbol {
        offset: 0,
        section: 2,
        _type: defs::STT_SECTION,
        bind: defs::STB_LOCAL,
        ..Default::default()
    };

    let mut result = vec![file, text, data];
    let last_idx = result.len();

    let mut relocations_result = vec![];
    let mut symbol_idxs = HashMap::new();
    for (idx, symbol) in symbols.iter().enumerate() {
        let _type = match symbol.get_type() {
            symbols::SymbolType::Data(DataSymbol::Comptime) => defs::STT_OBJECT,
            symbols::SymbolType::Text => defs::STT_FUNC,
            _ => continue,
        };

        symbol_idxs.insert(symbol.get_name(), last_idx + idx + 1);

        let section = match symbol.get_section() {
            symbols::Section::Undefined => SHN_UNDEF,
            symbols::Section::Text => 1,
            symbols::Section::Data => 2,
            symbols::Section::Absolute => SHN_ABS,
        };

        let bind = match symbol.get_scope() {
            symbols::SymbolScope::Local => defs::STB_LOCAL,
            symbols::SymbolScope::Global => defs::STB_GLOBAL,
        };

        let symbol = Symbol {
            name: symbol.get_name().to_string(),
            offset: symbol.get_offset() as u64,
            section,
            _type: _type,
            bind,
        };
        result.push(symbol);
    }

    let last_local_idx = result.len();
    dbg!(last_local_idx);

    for rel in relocations {
        let idx = match symbol_idxs.get(rel.get_symbol()) {
            Some(idx) => *idx,
            None => {
                let _type = match rel.get_type() {
                    symbols::SymbolType::Data(DataSymbol::Comptime) => defs::STT_OBJECT,
                    symbols::SymbolType::Text => defs::STT_FUNC,
                    _ => unreachable!(),
                };
                let symbol = Symbol {
                    name: rel.get_symbol().to_string(),
                    offset: 0,
                    section: SHN_UNDEF,
                    _type,
                    bind: defs::STB_GLOBAL,
                };
                let idx = result.len() + 1;
                symbol_idxs.insert(rel.get_symbol(), idx);

                result.push(symbol);
                idx
            }
        };
        let _type = match rel.get_type() {
            symbols::SymbolType::Data(DataSymbol::Comptime) => defs::STT_OBJECT,
            symbols::SymbolType::Text => defs::STT_FUNC,
            _ => continue,
        };

        relocations_result.push(Relocation::new(idx, rel.get_offset() as u64, _type));
    }

    let start = Symbol {
        name: "_start".to_string(),
        offset: 0,
        section: 1,
        _type: defs::STT_NOTYPE,
        bind: defs::STB_GLOBAL,
        ..Default::default()
    };

    result.push(start);

    (result, relocations_result, last_local_idx)
}

fn align(mut v: Vec<u8>, alignment: usize) -> Vec<u8> {
    let reminder = v.len() % alignment;
    let new_len = v.len() + alignment - reminder;
    v.resize(new_len, 0);
    v
}

// fn make_local_calls_before_global(
//     calls: &HashMap<String, Call>,
// ) -> (Vec<(&String, &Call)>, usize) {
//     let mut local_calls = vec![];
//     let mut global_calls = vec![];

//     for (name, call) in calls {
//         match call.call_type {
//             CallType::Local => local_calls.push((name, call)),
//             CallType::Global => global_calls.push((name, call)),
//         }
//     }
//     let size = local_calls.len();
//     ([local_calls, global_calls].concat(), size)
// }

// fn get_call_type(call_type: CallType) -> u8 {
//     match call_type {
//         CallType::Local => defs::STB_LOCAL,
//         CallType::Global => defs::STB_GLOBAL,
//     }
// }
