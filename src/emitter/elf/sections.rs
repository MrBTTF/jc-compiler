use std::{
    collections::{hash_map::Entry, BTreeMap, HashMap},
    mem,
};

use elf::section;

use crate::emitter::{
    ast, data,
    symbols::{DataSymbol, SymbolType},
    text::Sliceable,
};

use super::{defs, Data, DataRef, DataType};

pub type DWord = u64;

pub const VIRTUAL_ADDRESS_START: DWord = 0x08000000;
const DATA_SECTION_OFFSET: DWord =
    (mem::size_of::<ELFHeader>() + mem::size_of::<ProgramHeader>() * 3) as _;
pub const DATA_SECTION_ADDRESS_START: DWord = VIRTUAL_ADDRESS_START + DATA_SECTION_OFFSET;

#[derive(Debug)]
#[repr(C)]
pub struct ELFHeader {
    e_ident_magic_number: [u8; 4],
    e_ident_class: u8,
    e_ident_data: u8,
    e_ident_version: u8,
    e_ident_abi: u8,
    e_ident_abi_version: u8,
    e_ident_pad: [u8; 7],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: DWord,
    e_phoff: DWord,
    e_shoff: DWord,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

// impl ELFHeader {
//     pub fn get_entry_point(data_section_size: usize) -> u64 {
//         let e_ehsize = mem::size_of::<ELFHeader>() as u16;
//         let e_phentsize = mem::size_of::<ProgramHeader>() as u16;
//         let e_phnum = 3;
//         VIRTUAL_ADDRESS_START
//             + (e_ehsize + e_phentsize * e_phnum) as DWord
//             + data_section_size as DWord
//     }
// }

impl Sliceable for ELFHeader {}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct RelocationTable {
    r_offset: u64,
    r_info: u64,
    r_addend: i64,
}

impl Sliceable for RelocationTable {}

#[repr(C)]
pub struct ProgramHeader {
    p_type: u32,
    p_flags: u32,
    p_offset: DWord,
    p_vaddr: DWord,
    p_paddr: DWord,
    p_filesz: DWord,
    p_memsz: DWord,
    p_align: DWord,
}

impl Sliceable for ProgramHeader {}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct SectionHeader {
    sh_name: u32,
    sh_type: u32,
    sh_flags: DWord,
    sh_addr: DWord,
    sh_offset: DWord,
    sh_size: DWord,
    sh_link: u32,
    sh_info: u32,
    sh_addralign: DWord,
    sh_entsize: DWord,
}

impl SectionHeader {
    pub fn new(
        sh_size: usize,
        sh_type: u32,
        sh_flags: DWord,
        sh_addralign: DWord,
        sh_entsize: DWord,
        sh_info: u32,
        sh_link: u32,
    ) -> Self {
        Self {
            sh_size: sh_size as DWord,
            sh_type,
            sh_flags,
            sh_addralign,
            sh_entsize,
            sh_info,
            sh_link,
            sh_name: 0,
            sh_addr: 0,
            sh_offset: 0,
        }
    }
}

impl Sliceable for SectionHeader {}

#[repr(C)]
#[derive(Debug, Default)]
pub struct SymbolTable {
    st_name: u32,
    st_info: u8,
    st_other: u8,
    st_shndx: u16,
    st_value: u64,
    st_size: u64,
}

impl Sliceable for SymbolTable {}

pub fn build_header(sections: &[SectionHeader], shstrtab_index: usize) -> ELFHeader {
    let sections_size: u64 = sections.iter().map(|s| s.sh_size).sum();

    let e_ehsize = mem::size_of::<ELFHeader>() as u16;
    let e_shoff = e_ehsize as DWord;
    let e_shentsize = mem::size_of::<SectionHeader>() as u16;
    ELFHeader {
        e_ident_magic_number: defs::ELF_MAGIC,
        e_ident_class: defs::ELF_CLASS_64_BIT,
        e_ident_data: defs::ELF_DATA_LITTLE_ENDIAN,
        e_ident_version: defs::ELF_VERSION_CURRENT,
        e_ident_abi: defs::ELF_OSABI_SYSTEM_V,
        e_ident_abi_version: defs::ELF_ABI_VERSION_NONE,
        e_ident_pad: [0; 7],
        e_type: defs::ELF_TYPE_REL,
        e_machine: defs::ELF_MACHINE_X86_64,
        e_version: defs::ELF_VERSION_CURRENT as u32,
        e_entry: 0x0,
        e_phoff: 0x0,
        e_shoff,
        e_flags: 0x0,
        e_ehsize,
        e_phentsize: 0x0,
        e_phnum: 0x0,
        e_shentsize,
        e_shnum: 1 + sections.len() as u16,
        e_shstrndx: shstrtab_index as u16,
    }
}

pub fn build_section_headers(sections: &[SectionHeader], section_names: &[&str]) -> Vec<u8> {
    let mut result = SectionHeader::default().as_vec();

    let mut sh_offset = (mem::size_of::<ELFHeader>()
        + mem::size_of::<SectionHeader>() * (sections.len() + 1)) as DWord;
    let mut sh_name = 0x1;

    for (i, section) in sections.iter().enumerate() {
        let mut section = *section;
        section.sh_name = sh_name;
        section.sh_offset = sh_offset;

        sh_name += section_names[i].len() as u32 + 1;
        // let mut remainder = section.sh_size % section.sh_addralign;
        // if section.sh_addralign < 2 {
        //     remainder = section.sh_addralign;
        // }
        // sh_offset += section.sh_size + section.sh_addralign - remainder;
        sh_offset += section.sh_size;

        result.extend(section.as_vec());
    }
    result
}

pub fn build_symtab_section_header() -> SectionHeader {
    SectionHeader {
        sh_name: 0x17,
        sh_type: defs::SEGMENT_TYPE_SYMTAB,
        sh_flags: defs::SEGMENT_FLAGS_NONE,
        sh_addr: 0x0,
        sh_offset: 0x01e0,
        sh_size: 0x70,
        sh_link: 0x05,
        sh_info: 0x06,
        sh_addralign: 0x04,
        sh_entsize: 0x10,
    }
}

pub fn build_strtab_section_header() -> SectionHeader {
    SectionHeader {
        sh_name: 0x1f,
        sh_type: defs::SEGMENT_TYPE_STRTAB,
        sh_flags: defs::SEGMENT_FLAGS_NONE,
        sh_addr: 0x0,
        sh_offset: 0x0250,
        sh_size: 0x1e,
        sh_link: 0x00,
        sh_info: 0x00,
        sh_addralign: 0x01,
        sh_entsize: 0x00,
    }
}

pub fn build_rel_text_section_header() -> SectionHeader {
    SectionHeader {
        sh_name: 0x27,
        sh_type: defs::SEGMENT_TYPE_REL,
        sh_flags: defs::SEGMENT_FLAGS_NONE,
        sh_addr: 0x0,
        sh_offset: 0x0270,
        sh_size: 0x08,
        sh_link: 0x04,
        sh_info: 0x01,
        sh_addralign: 0x04,
        sh_entsize: 0x08,
    }
}

#[rustfmt::skip]
pub fn build_shstrtab_section(section_names: &[&str]) -> Vec<u8> {
    section_names.iter().fold(vec![0], |mut acc, name| {
        acc.extend(name.as_bytes());
        acc.push(0);
        acc
    })
    // [
    //     vec![0],
    //     CString::new(".text").unwrap().into_bytes_with_nul(),
    //     CString::new(".data").unwrap().into_bytes_with_nul(),
    //     CString::new(".shstrtab").unwrap().into_bytes_with_nul(),
    // ].concat()
}

#[derive(Debug, Default, Clone)]
pub struct Symbol {
    pub name: String,
    pub offset: u64,
    pub section: u16,
    pub _type: u8,
    pub bind: u8,
}

pub struct SymStr {
    symtab: Vec<u8>,
    strtab: Vec<u8>,
}

impl SymStr {
    pub fn new(symbols: &[Symbol]) -> Self {
        let mut symtab = SymbolTable::default().as_vec();
        let mut strtab = vec![0];

        for symbol in symbols {
            let st_name = if symbol._type == defs::STT_SECTION {
                0
            } else {
                let name = strtab.len();
                strtab.extend(symbol.name.as_bytes());
                strtab.push(0);
                name as u32
            };
            let st_info = (symbol.bind << 4) + symbol._type;

            let st_shndx = symbol.section;

            let symbol_table = SymbolTable {
                st_name: st_name,
                st_value: symbol.offset,
                st_size: 0x0,
                st_info: st_info,
                st_other: 0x0,
                st_shndx: st_shndx,
            };
            symtab.extend(symbol_table.as_vec())
        }
        Self { symtab, strtab }
    }

    pub fn get_symtab(&self) -> &[u8] {
        self.symtab.as_slice()
    }

    pub fn get_strtab(&self) -> &[u8] {
        self.strtab.as_slice()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Relocation {
    symbol: u64,
    offset: u64,
    _type: u8,
}

impl Relocation {
    pub fn new(symbol: usize, offset: u64, _type: u8) -> Self {
        Self {
            symbol: symbol as u64,
            offset: offset,
            _type,
        }
    }
}

pub fn build_rel_text_section(relocations: &[Relocation]) -> Vec<u8> {
    let mut result = vec![];
    for rel in relocations {
        let (r_info, r_addend) = match rel._type {
            defs::STT_OBJECT => {
                let sym = (rel.symbol) << 32;
                let _type = 1;
                let info = sym + _type;
                (info, 0x0)
            }
            defs::STT_FUNC => {
                let sym = (rel.symbol) << 32;
                let _type = 4;
                let info = sym + _type;
                (info, -0x4) // Next pc address minus value address in CALL
            }
            _ => continue,
        };

        result.extend(
            RelocationTable {
                r_offset: rel.offset,
                r_info: r_info,
                r_addend: r_addend,
            }
            .as_vec(),
        )
    }
    result
}

pub fn build_data_section(literals: &HashMap<String, Data>) -> Vec<u8> {
    let mut literals: Vec<_> = literals
        .iter()
        .filter_map(|(id, data)| match data.decl_type {
            ast::VarDeclarationType::Let => None,
            ast::VarDeclarationType::Const => {
                Some((data.data_loc, id.clone(), data.data_type.clone()))
            }
        })
        .collect();
    literals.sort_by_key(|(data_loc, _, _)| *data_loc);
    literals
        .iter()
        .fold(vec![], |mut acc, (_, _, lit)| match lit {
            DataType::String(string) => {
                acc.extend(string.clone().into_bytes());
                acc
            }
            DataType::Int(n) => {
                acc.extend(n.to_le_bytes().to_vec());
                acc
            }
        })
}
