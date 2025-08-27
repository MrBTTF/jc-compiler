mod defs;
pub mod sections;

use self::defs::*;
use crate::emitter::text::{CodeContext, Sliceable};
use sections::*;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::process::Command;
use std::{collections::BTreeMap, env, fs, io::Write, mem, time::SystemTime};

use super::{ast, symbols};

pub fn build(
    output_path: PathBuf,
    code_context: &CodeContext,
    symbols: &[symbols::Symbol],
    relocations: &[symbols::Relocation],
) {
    let object_file = output_path.with_extension("obj");

    let mut debug_code_file =
        fs::File::create(env::current_dir().unwrap().join("local/debug/code.txt")).unwrap();
    debug_code_file
        .write_all(format!("{:#?}", &code_context).as_bytes())
        .unwrap();

    let created_at = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;

    let data_section_data = build_data_section(&symbols);

    let (symbol_table, string_table_data, relocations_information) =
        build_symbol_table(symbols, relocations);

    let mut section_layout = SectionLayout::new(vec![
        Section::new(
            ".text".to_string(),
            code_context.get_code_size() as u32,
            SECTION_CHARACTERISTICS_TEXT
                | SECTION_CHARACTERISTICS_EXEC
                | SECTION_CHARACTERISTICS_READ
                | SECTION_CHARACTERISTICS_ALIGN_16BYTES,
        ),
        Section::new(
            ".data".to_string(),
            data_section_data.len() as u32,
            SECTION_CHARACTERISTICS_DATA
                | SECTION_CHARACTERISTICS_READ
                | SECTION_CHARACTERISTICS_WRITE
                | SECTION_CHARACTERISTICS_ALIGN_4BYTES,
        ),
    ]);

    let relocations_information_data =
        build_relocation_section(&mut section_layout, relocations_information);

    let text_section = section_layout.get_section(".text");
    let data_section = section_layout.get_section(".data");

    let text_section_data: Vec<u8> = code_context.to_bin();

    let symbol_table_data = symbol_table.iter().fold(vec![], |mut acc, s| {
        acc.extend(s.as_vec());
        acc
    });

    let file_header = build_file_header(
        created_at,
        section_layout,
        relocations_information_data.len() as u32,
        symbol_table.len() as u32,
    );

    let headers = [
        file_header,
        text_section.get_header(),
        data_section.get_header(),
    ]
    .concat();

    let mut file = fs::File::create(&object_file).unwrap();
    write_all_aligned(&mut file, &headers).unwrap();
    write_all_aligned(&mut file, &text_section_data).unwrap();
    write_all_aligned(&mut file, &data_section_data).unwrap();
    write_all_aligned(&mut file, &relocations_information_data).unwrap();
    write_all_aligned(&mut file, &symbol_table_data).unwrap();
    write_all_aligned(&mut file, &string_table_data).unwrap();

    let child = Command::new("lld-link")
        .args(&[
            object_file.to_str().unwrap(),
            "local/libs/kernel32.lib",
            "local/libs/ucrt.lib",
            "/subsystem:console",
            "/nodefaultlib",
            "/entry:_start",
            &format!("/out:{}", output_path.to_str().unwrap()),
        ])
        .output()
        .unwrap();

    if !child.status.success() {
        panic!("{}", String::from_utf8(child.stderr).unwrap());
    }
}

fn write_all_aligned(file: &mut fs::File, buf: &[u8]) -> Result<(), std::io::Error> {
    // let buf = get_data_aligned(buf.to_vec());
    file.write_all(&buf)
}

fn build_symbol_table(
    symbols: &[symbols::Symbol],
    relocations: &[symbols::Relocation],
) -> (
    Vec<Symbol>,
    Vec<u8>,
    HashMap<String, Vec<RelocationInformation>>,
) {
    let mut symbol_table = vec![];
    let mut symbol_idxs = HashMap::new();
    let mut string_table = vec![];
    let mut relocation_information = HashMap::new();

    let start_symbol = symbols::Symbol::new(
        "_start".to_string(),
        0,
        symbols::Section::Text,
        symbols::SymbolType::Text,
        symbols::SymbolScope::Global,
        vec![],
    );

    let text_symbol = symbols::Symbol::new(
        ".text".to_string(),
        0,
        symbols::Section::Text,
        symbols::SymbolType::Text,
        symbols::SymbolScope::Global,
        vec![],
    );

    let data_symbol = symbols::Symbol::new(
        ".data".to_string(),
        0,
        symbols::Section::Data,
        symbols::SymbolType::Text,
        symbols::SymbolScope::Global,
        vec![],
    );

    let symbols = &[&[start_symbol, text_symbol, data_symbol], symbols].concat();

    for symbol in symbols {
        let name: SymbolNameOrOffset = if symbol.get_name().len() > 8 {
            let offset = string_table.len() as u32 + 4; // First 4 bytes is the size of the string table
            string_table.extend([symbol.get_name().as_bytes(), &[0]].concat());
            SymbolNameOrOffset::from_offset(offset)
        } else {
            SymbolNameOrOffset::new(symbol.get_name())
        };
        let value = symbol.get_offset() as u32;
        let section = match symbol.get_section() {
            symbols::Section::Undefined => SYMBOL_SECTION_UNDEFINED,
            symbols::Section::Text => {
                symbol_idxs.insert(symbol.get_name(), (symbol_idxs.len(), ".text".to_string()));
                SYMBOL_SECTION_TEXT
            }
            symbols::Section::Data => {
                symbol_idxs.insert(symbol.get_name(), (symbol_idxs.len(), ".text".to_string()));
                SYMBOL_SECTION_DATA
            }
            symbols::Section::Absolute => SYMBOL_SECTION_ABSOLUTE,
        };
        let storage_class = match symbol.get_type() {
            symbols::SymbolType::Data(data_symbol) => match data_symbol {
                symbols::DataSymbol::Comptime => SYMBOL_STORAGE_CLASS_STATIC,
                symbols::DataSymbol::Runtime => SYMBOL_STORAGE_CLASS_STATIC,
            },
            symbols::SymbolType::Text => {
                if symbol.get_name() == "_start" {
                    SYMBOL_STORAGE_CLASS_EXTERNAL
                } else {
                    SYMBOL_STORAGE_CLASS_STATIC
                }
            }
        };

        symbol_table.push(Symbol::new(name, value, section, storage_class));
    }

    let mut last_symbol_idx = symbol_idxs.len();
    for rel in relocations {
        let (symbol_table_index, section) = match symbol_idxs.get(rel.get_symbol()) {
            Some(s) => s,
            None => {
                let name: SymbolNameOrOffset = if rel.get_symbol().len() > 8 {
                    let offset = string_table.len() as u32 + 4; // First 4 bytes is the size of the string table
                    string_table.extend([rel.get_symbol().as_bytes(), &[0]].concat());
                    SymbolNameOrOffset::from_offset(offset)
                } else {
                    SymbolNameOrOffset::new(rel.get_symbol())
                };
                let value = 0;
                let section = match rel.get_type() {
                    symbols::SymbolType::Data(_) => SYMBOL_SECTION_DATA,
                    symbols::SymbolType::Text => SYMBOL_SECTION_UNDEFINED,
                };
                let storage_class = SYMBOL_STORAGE_CLASS_EXTERNAL;

                symbol_table.push(Symbol::new(name, value, section, storage_class));
                symbol_idxs.insert(rel.get_symbol(), (symbol_idxs.len(), ".text".to_string()));

                let idx = last_symbol_idx;
                last_symbol_idx += 1;
                &(idx, ".text".to_string())
            }
        };

        // if rel.get_type() == symbols::SymbolType::Text {
        //     continue;
        // }
        let rels_per_sections = relocation_information
            .entry(section.clone())
            .or_insert(vec![]);

        let rel_type = match rel.get_type() {
            symbols::SymbolType::Data(data_symbol) => match data_symbol {
                symbols::DataSymbol::Comptime => RELOCATION_INFORMATION_TYPE_ADDR64,
                symbols::DataSymbol::Runtime => RELOCATION_INFORMATION_TYPE_32,
            },
            symbols::SymbolType::Text => RELOCATION_INFORMATION_TYPE_32,
        };

        rels_per_sections.push(RelocationInformation::new(
            rel.get_offset() as u32,
            *symbol_table_index as u32,
            rel_type,
        ));
    }

    let string_table_size = string_table.len() as u32 + 4;
    string_table = [string_table_size.to_le_bytes().to_vec(), string_table].concat();

    (symbol_table, string_table, relocation_information)
}
