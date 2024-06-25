use std::{collections::BTreeMap, ffi::CString, mem};

use crate::emitter::{ast, code_context::Sliceable};

use super::{defs, Data};

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

impl ELFHeader {
    pub fn get_entry_point(data_section_size: usize) -> u64 {
        let e_ehsize = mem::size_of::<ELFHeader>() as u16;
        let e_phentsize = mem::size_of::<ProgramHeader>() as u16;
        let e_phnum = 3;
        VIRTUAL_ADDRESS_START
            + (e_ehsize + e_phentsize * e_phnum) as DWord
            + data_section_size as DWord
    }
}

impl Sliceable for ELFHeader {}

#[repr(C)]
pub struct RelocationTable {
    r_offset: u32,
    r_info: u32,
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

impl Sliceable for SectionHeader {}

#[repr(C)]
pub struct SymbolTable {
    st_name: u32,
    st_value: u32,
    st_size: u32,
    st_info: u8,
    st_other: u8,
    st_shndx: u16,
}

impl Sliceable for SymbolTable {}

pub fn build_header(data_section_size: usize, section_entry: usize) -> ELFHeader {
    let e_ehsize = mem::size_of::<ELFHeader>() as u16;
    let e_phoff = e_ehsize as DWord;
    let e_shoff = e_phoff + section_entry as DWord;
    let e_phentsize = mem::size_of::<ProgramHeader>() as u16;
    let e_phnum = 3;
    let e_shentsize = mem::size_of::<SectionHeader>() as u16;
    ELFHeader {
        e_ident_magic_number: defs::ELF_MAGIC,
        e_ident_class: defs::ELF_CLASS_64_BIT,
        e_ident_data: defs::ELF_DATA_LITTLE_ENDIAN,
        e_ident_version: defs::ELF_VERSION_CURRENT,
        e_ident_abi: defs::ELF_OSABI_SYSTEM_V,
        e_ident_abi_version: defs::ELF_ABI_VERSION_NONE,
        e_ident_pad: [0; 7],
        e_type: defs::ELF_TYPE_EXEC,
        e_machine: defs::ELF_MACHINE_X86_64,
        e_version: defs::ELF_VERSION_CURRENT as u32,
        e_entry: VIRTUAL_ADDRESS_START
            + (e_ehsize + e_phentsize * e_phnum) as DWord
            + data_section_size as DWord,
        e_phoff,
        e_shoff,
        e_flags: 0x0,
        e_ehsize,
        e_phentsize,
        e_phnum,
        e_shentsize,
        e_shnum: 4,
        e_shstrndx: 3,
    }
}

pub fn build_program_headers(text_section_size: usize, data_section_size: usize) -> Vec<u8> {
    let null_p_filesz = (mem::size_of::<ELFHeader>() + mem::size_of::<ProgramHeader>() * 3)
        .try_into()
        .unwrap();
    let null_program_header = ProgramHeader {
        p_type: defs::PROGRAM_TYPE_LOAD,
        p_offset: 0x0,
        p_vaddr: VIRTUAL_ADDRESS_START,
        p_paddr: VIRTUAL_ADDRESS_START,
        p_filesz: null_p_filesz,
        p_memsz: null_p_filesz,
        p_flags: defs::PROGRAM_HEADER_READ,
        p_align: 1,
    };
    let data_p_filesz = data_section_size as DWord;
    let data_program_header = ProgramHeader {
        p_type: defs::PROGRAM_TYPE_LOAD,
        p_flags: defs::PROGRAM_HEADER_READ | defs::PROGRAM_HEADER_WRITE,
        p_offset: null_p_filesz,
        p_vaddr: VIRTUAL_ADDRESS_START + null_p_filesz,
        p_paddr: VIRTUAL_ADDRESS_START + null_p_filesz,
        p_filesz: data_p_filesz,
        p_memsz: data_p_filesz,
        p_align: 1,
    };
    let text_p_filesz = text_section_size as DWord;
    let text_program_header = ProgramHeader {
        p_type: defs::PROGRAM_TYPE_LOAD,
        p_offset: data_program_header.p_offset + data_p_filesz,
        p_vaddr: data_program_header.p_vaddr + data_p_filesz,
        p_paddr: data_program_header.p_vaddr + data_p_filesz,
        p_filesz: text_p_filesz,
        p_memsz: text_p_filesz,
        p_flags: defs::PROGRAM_HEADER_READ | defs::PROGRAM_HEADER_EXEC,
        p_align: 1,
    };
    [
        null_program_header.as_slice(),
        data_program_header.as_slice(),
        text_program_header.as_slice(),
    ]
    .concat()
}

pub fn build_section_headers(
    text_section_size: usize,
    data_section_size: usize,
    shstrtab_section_size: usize,
) -> Vec<u8> {
    let data_entry: DWord = (mem::size_of::<ELFHeader>() + mem::size_of::<ProgramHeader>() * 3)
        .try_into()
        .unwrap();
    let null_section_header = SectionHeader {
        sh_name: 0x0,
        sh_type: defs::SEGMENT_TYPE_NULL,
        sh_flags: defs::SEGMENT_FLAGS_NONE,
        sh_addr: 0x0,
        sh_offset: 0x0,
        sh_size: 0x0,
        sh_link: 0x0,
        sh_info: 0x0,
        sh_addralign: 0x0,
        sh_entsize: 0x0,
    };
    let data_section_header = SectionHeader {
        sh_name: 0x11,
        sh_type: defs::SEGMENT_TYPE_PROGBITS,
        sh_flags: defs::SEGMENT_FLAGS_WRITE | defs::SEGMENT_FLAGS_ALLOC,
        sh_addr: VIRTUAL_ADDRESS_START + data_entry,
        sh_offset: data_entry,
        sh_size: data_section_size as DWord,
        sh_link: 0x0,
        sh_info: 0x0,
        sh_addralign: 0x0,
        sh_entsize: 0x0,
    };
    let text_section_header = SectionHeader {
        sh_name: 0x0B,
        sh_type: defs::SEGMENT_TYPE_PROGBITS,
        sh_flags: defs::SEGMENT_FLAGS_ALLOC | defs::SEGMENT_FLAGS_EXECINSTR,
        sh_addr: data_section_header.sh_addr + data_section_header.sh_size,
        sh_offset: data_entry + data_section_header.sh_size,
        sh_size: text_section_size as DWord,
        sh_link: 0x0,
        sh_info: 0x0,
        sh_addralign: 0x0,
        sh_entsize: 0x0,
    };
    let shstrtab_section_header = SectionHeader {
        sh_name: 0x01,
        sh_type: defs::SEGMENT_TYPE_STRTAB,
        sh_flags: defs::SEGMENT_FLAGS_NONE,
        sh_addr: text_section_header.sh_addr + text_section_header.sh_size,
        sh_offset: text_section_header.sh_offset + text_section_header.sh_size,
        sh_size: shstrtab_section_size as DWord,
        sh_link: 0x0,
        sh_info: 0x0,
        sh_addralign: 0x0,
        sh_entsize: 0x0,
    };
    [
        null_section_header.as_slice(),
        data_section_header.as_slice(),
        text_section_header.as_slice(),
        shstrtab_section_header.as_slice(),
    ]
    .concat()
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
pub fn build_shstrtab_section() -> Vec<u8> {
    [
        vec![0],
        CString::new(".shstrtab").unwrap().into_bytes_with_nul(),
        CString::new(".text").unwrap().into_bytes_with_nul(),
        CString::new(".data").unwrap().into_bytes_with_nul(),
    ].concat()
}

pub fn build_symtab_sections() -> Vec<u8> {
    let t0 = SymbolTable {
        st_name: 0x0,
        st_value: 0x0,
        st_size: 0x0,
        st_info: 0x0,
        st_other: 0x0,
        st_shndx: 0x0,
    };
    let t1 = SymbolTable {
        st_name: 0x1,
        st_value: 0x0,
        st_size: 0x0,
        st_info: 0x4,
        st_other: 0x0,
        st_shndx: 0xfff1,
    };
    let t2 = SymbolTable {
        st_name: 0x0,
        st_value: 0x0,
        st_size: 0x0,
        st_info: 0x3,
        st_other: 0x0,
        st_shndx: 0x01,
    };

    let t3 = SymbolTable {
        st_name: 0x0,
        st_value: 0x0,
        st_size: 0x0,
        st_info: 0x3,
        st_other: 0x0,
        st_shndx: 0x02,
    };
    let t4 = SymbolTable {
        st_name: 0x0F,
        st_value: 0x0,
        st_size: 0x0,
        st_info: 0x0,
        st_other: 0x0,
        st_shndx: 0x02,
    };
    let t5 = SymbolTable {
        st_name: 0x13,
        st_value: 0x0E,
        st_size: 0x0,
        st_info: 0x0,
        st_other: 0x0,
        st_shndx: 0xfff1,
    };
    let t6 = SymbolTable {
        st_name: 0x17,
        st_value: 0x00,
        st_size: 0x0,
        st_info: 0x10,
        st_other: 0x0,
        st_shndx: 0x01,
    };
    [
        t0.as_slice(),
        t1.as_slice(),
        t2.as_slice(),
        t3.as_slice(),
        t4.as_slice(),
        t5.as_slice(),
        t6.as_slice(),
    ]
    .concat()
}

#[rustfmt::skip]
pub fn build_strtab_section() -> Vec<u8> {
    [
        vec![0x0_u8],
        "src/hello.asm".into(),
        vec![0x0_u8],
        "msg".into(),
        vec![0x0_u8],
        "len".into(),
        vec![0x0_u8],
        "_start".into(),
        vec![0x0_u8],
    ].concat()
}

pub fn build_rel_text_section() -> RelocationTable {
    RelocationTable {
        r_offset: 0x06,
        r_info: 0x301,
    }
}

pub fn build_data_section(literals: BTreeMap<ast::Ident, Data>) -> Vec<u8> {
    let mut literals: Vec<_> = literals
        .iter()
        .filter_map(|(id, data)| match data.decl_type {
            ast::DeclarationType::Let => None,
            ast::DeclarationType::Const => Some((data.data_loc(), id.clone(), data.lit.clone())),
        })
        .collect();
    literals.sort_by_key(|(data_loc, _, _)| *data_loc);
    literals
        .iter()
        .fold(vec![], |mut acc, (_, _, lit)| match lit {
            ast::Literal::String(string) => {
                acc.extend(string.clone().into_bytes());
                acc
            }
            ast::Literal::Number(n) => {
                acc.extend(n.value.to_le_bytes().to_vec());
                acc
            }
        })
}
