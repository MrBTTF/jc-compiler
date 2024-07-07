pub mod defs;
pub mod sections;

use std::{
    collections::{hash_map::Entry, BTreeMap, HashMap},
    fmt::Result,
    fs,
    io::Write,
    mem,
    path::PathBuf,
    process::Command,
};

use elf::relocation::Rel;

use self::{
    super::{ast, data::*},
    sections::*,
};

use super::{
    code_context::{self, CodeContext, Sliceable},
    Emitter, Ident,
};

pub fn build(output_path: PathBuf, emitter: Emitter, variables: &BTreeMap<ast::Ident, Data>) {
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
    let (symbols, relocations, last_local_idx) =
        build_symbols("local/bin/hello.o", &code_context, &variables);
    let symstr = SymStr::new(symbols.as_slice());
    let symtab_section_data = symstr.get_symtab();
    let strtab_section_data = symstr.get_strtab();
    let relocation_section_data: Vec<u8> = build_rel_text_section(&relocations);

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

    // let program_headers = &build_program_headers(text_section.len(), data_section.len());
    let header = &build_header(section_headers, 3);

    let header_data = header.as_slice();

    let object_file = output_path.with_extension("o");

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
    code_context: &CodeContext,
    variables: &BTreeMap<ast::Ident, Data>,
) -> (Vec<Symbol>, Vec<Relocation>, usize) {
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
    let mut result = vec![file, text, data];

    let mut relocations = vec![];
    for (pc, d) in code_context.get_const_data() {
        let offset = variables
            .get(&Ident {
                value: d.symbol.to_string(),
            })
            .unwrap()
            .data_loc;
        let data_loc = (code_context.get_offset(pc) + d.offset) as u64;
        let symbol = Symbol {
            name: d.symbol.to_string(),
            offset,
            data_loc: data_loc,
            section: Some(2),
            _type: defs::STT_OBJECT,
            bind: defs::STB_LOCAL,
            ..Default::default()
        };
        relocations.push(Relocation::new(result.len(), data_loc, symbol._type));
        result.push(symbol.clone());
    }

    let last_local_idx = result.len();
    dbg!(last_local_idx);

    let calls = code_context.get_calls();
    for (name, call) in calls {
        let mut existing_symbols: HashMap<String, usize> = HashMap::new();

        for pc in &call.offsets {
            let address_loc = code_context.get_offset(*pc) + 1;
            dbg!(&call, address_loc);

            let symbol: Symbol = Symbol {
                name: name.clone(),
                data_loc: address_loc as u64,
                offset: 0,
                section: Some(0),
                _type: defs::STT_FUNC,
                bind: defs::STB_GLOBAL,
                ..Default::default()
            };

            if let Entry::Occupied(s) = existing_symbols.entry(name.clone()) {
                relocations.push(Relocation::new(*s.get(), address_loc as u64, symbol._type));
                continue;
            }

            existing_symbols.insert(symbol.name.to_string(), result.len());
            relocations.push(Relocation::new(result.len(), symbol.data_loc, symbol._type));
            result.push(symbol.clone());
        }
    }

    let start = Symbol {
        name: "_start".to_string(),
        offset: 0,
        section: Some(1),
        _type: defs::STT_NOTYPE,
        bind: defs::STB_GLOBAL,
        ..Default::default()
    };

    result.push(start);

    (result, relocations, last_local_idx)
}

fn align(mut v: Vec<u8>, alignment: usize) -> Vec<u8> {
    let reminder = v.len() % alignment;
    let new_len = v.len() + alignment - reminder;
    v.resize(new_len, 0);
    v
}
