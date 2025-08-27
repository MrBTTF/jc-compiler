use elf::relocation;

use super::defs::*;
use crate::emitter::symbols;
use crate::emitter::text::Sliceable;
use std::{
    collections::{BTreeMap, HashMap},
    fmt::Debug,
    mem, usize, vec,
};

pub const BASE_OF_CODE: u32 = 0x1000;
pub const FILE_ALIGNMENT: u32 = 0x200;
pub const SECTION_ALIGNMENT: u32 = 0x1000;

pub fn round_to_multiple(num: u32, multiple: u32) -> u32 {
    if num == 0 {
        return 0;
    }
    multiple * ((num - 1) / multiple) + multiple
}

const DOS_STUB: [u8; 56] = [
    0x0E, 0x1F, 0xBA, 0x0E, 0x00, 0xB4, 0x09, 0xCD, 0x21, 0xB8, 0x01, 0x4C, 0xCD, 0x21, 0x54, 0x68,
    0x69, 0x73, 0x20, 0x70, 0x72, 0x6F, 0x67, 0x72, 0x61, 0x6D, 0x20, 0x63, 0x61, 0x6E, 0x6E, 0x6F,
    0x74, 0x20, 0x62, 0x65, 0x20, 0x72, 0x75, 0x6E, 0x20, 0x69, 0x6E, 0x20, 0x44, 0x4F, 0x53, 0x20,
    0x6D, 0x6F, 0x64, 0x65, 0x2E, 0x24, 0x00, 0x00,
];

const IMAGE_SIZEOF_SHORT_NAME: usize = 8;

pub const IMAGE_BASE: u64 = 0x140000000;

pub fn get_data_aligned(mut data: Vec<u8>) -> Vec<u8> {
    let diff = round_to_multiple(data.len() as u32, FILE_ALIGNMENT) - data.len() as u32;
    data.extend(std::iter::repeat(0x0).take(diff as usize));
    data
}

#[derive(Debug, Clone)]
pub struct Section {
    name: String,
    pointer_to_raw_data: u32,
    pointer_to_relocations: u32,
    number_of_relocations: u16,
    pub size_of_raw_data: u32,
    characteristics: u32,
}

impl Section {
    pub fn new(name: String, data_size: u32, characteristics: u32) -> Self {
        Self {
            name,
            characteristics,
            pointer_to_raw_data: 0,
            pointer_to_relocations: 0,
            number_of_relocations: 0,
            size_of_raw_data: data_size,
        }
    }

    pub fn get_header(&self) -> Vec<u8> {
        build_section_header(
            &self.name,
            self.pointer_to_raw_data,
            self.pointer_to_relocations,
            self.number_of_relocations,
            self.size_of_raw_data,
            self.characteristics,
        )
    }
}
#[derive(Debug)]
pub struct SectionLayout {
    sections: BTreeMap<String, Section>,
    size_of_headers: u32,
    text_size: u32,
    data_size: u32,
    bss_size: u32,
}

impl SectionLayout {
    pub fn new(sections: Vec<Section>) -> Self {
        let size_of_section_header = sections.len() as u32 * mem::size_of::<SectionHeader>() as u32;

        let size_of_headers = size_of_section_header;

        let mut sections_end = mem::size_of::<FileHeader>() as u32 + size_of_headers;

        let mut text_size = 0;
        let mut data_size = 0;
        let bss_size = 0;
        let mut _sections = BTreeMap::new();
        for (i, section) in sections.iter().enumerate() {
            let mut section = section.clone();
            section.pointer_to_raw_data = sections_end;

            if section.characteristics & SECTION_CHARACTERISTICS_TEXT != 0 {
                text_size += section.size_of_raw_data;
            } else if section.characteristics & SECTION_CHARACTERISTICS_DATA != 0 {
                data_size += section.size_of_raw_data;
            }

            sections_end = sections_end + section.size_of_raw_data;

            _sections.insert(section.name.clone(), section);
        }
        // println!("{:#?}", _sections);
        Self {
            sections: _sections,
            size_of_headers,
            text_size,
            data_size,
            bss_size,
        }
    }

    pub fn get_section(&self, name: &str) -> Section {
        self.sections.get(name).unwrap().clone()
    }

    pub fn get_sections_end(&self) -> u32 {
        mem::size_of::<FileHeader>() as u32
            + self.size_of_headers
            + self
                .sections
                .values()
                .map(|s| s.size_of_raw_data)
                .sum::<u32>()
    }

    pub fn set_section(&mut self, name: &str, section: Section) {
        self.sections
            .entry(name.to_string())
            .and_modify(|s| *s = section);
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct FileHeader {
    machine: u16,
    number_of_sections: u16,
    time_date_stamp: u32,
    pointer_to_symbol_table: u32,
    number_of_symbols: u32,
    size_of_optional_header: u16,
    characteristics: u16,
}

impl Sliceable for FileHeader {}

#[derive(Debug)]
#[repr(C)]
pub struct SectionHeader {
    name: [u8; IMAGE_SIZEOF_SHORT_NAME],
    virtual_size: u32,
    virtual_address: u32,
    size_of_raw_data: u32,
    pointer_to_raw_data: u32,
    pointer_to_relocations: u32,
    pointer_to_linenumbers: u32,
    number_of_relocations: u16,
    number_of_linenumbers: u16,
    characteristics: u32,
}

impl Sliceable for SectionHeader {}

#[repr(C)]
pub union SymbolNameOrOffset {
    name: [u8; 8],
    offset: u64,
}

impl Sliceable for SymbolNameOrOffset {}

impl SymbolNameOrOffset {
    pub fn new(name: &str) -> Self {
        assert!(name.len() <= 8, "Symbol name too long: {}", name);
        let mut name_bytes = [0; 8];
        let bytes = name.as_bytes();
        name_bytes[..bytes.len()].copy_from_slice(bytes);
        Self { name: name_bytes }
    }

    pub fn from_offset(offset: u32) -> Self {
        Self {
            offset: (offset as u64) << 32,
        }
    }

    pub fn is_offset(&self) -> bool {
        unsafe { self.offset & 0x0000FFFF == 0 }
    }
}
impl Debug for SymbolNameOrOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            if self.is_offset() {
                write!(f, "Offset: {}", self.offset & 0xFFFF0000)
            } else {
                write!(f, "Name: {}", String::from_utf8_lossy(&self.name))
            }
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Symbol {
    name: SymbolNameOrOffset,
    value: u32,
    section: i16,
    _type: u16,
    storage_class: u8,
    aux: u8,
}

impl Sliceable for Symbol {
    fn as_vec(&self) -> Vec<u8> {
        [
            self.name.as_slice(),
            &self.value.to_le_bytes(),
            &self.section.to_le_bytes(),
            &self._type.to_le_bytes(),
            &self.storage_class.to_le_bytes(),
            &self.aux.to_le_bytes(),
        ]
        .concat()
    }
}

impl Symbol {
    pub fn new(name: SymbolNameOrOffset, value: u32, section: i16, storage_class: u8) -> Self {
        Symbol {
            name,
            value,
            section,
            _type: 0, // data type, irrelevant for now
            storage_class,
            aux: 0,
        }
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct RelocationInformation {
    virtual_address: u32,
    symbol_table_index: u32,
    _type: u16,
}

impl Sliceable for RelocationInformation {}

impl RelocationInformation {
    pub fn new(virtual_address: u32, symbol_table_index: u32, _type: u16) -> Self {
        Self {
            virtual_address,
            symbol_table_index,
            _type,
        }
    }
}

pub fn build_file_header(
    created_at: u32,
    section_layout: SectionLayout,
    relocation_table_size: u32,
    number_of_symbols: u32,
) -> Vec<u8> {
    FileHeader {
        machine: IMAGE_FILE_MACHINE_AMD64,
        number_of_sections: section_layout.sections.len() as u16,
        time_date_stamp: created_at,
        pointer_to_symbol_table: section_layout.get_sections_end() + relocation_table_size,
        number_of_symbols,
        size_of_optional_header: 0, // 0 for object files
        characteristics: IMAGE_FILE_LARGE_ADDRESS_AWARE,
    }
    .as_vec()
}

pub fn build_section_header(
    name: &str,
    pointer_to_raw_data: u32,
    pointer_to_relocations: u32,
    number_of_relocations: u16,
    size_of_raw_data: u32,
    characteristics: u32,
) -> Vec<u8> {
    let name_bytes = name.as_bytes();
    let mut name = [0; 8];
    name[..name_bytes.len()].copy_from_slice(name_bytes);
    SectionHeader {
        name,
        virtual_size: 0,
        virtual_address: 0,
        size_of_raw_data,
        pointer_to_raw_data,
        pointer_to_relocations,
        pointer_to_linenumbers: 0x0,
        number_of_relocations,
        number_of_linenumbers: 0x0,
        characteristics,
    }
    .as_vec()
}

pub fn build_data_section(symbols: &[symbols::Symbol]) -> Vec<u8> {
    if symbols.len() == 0 {
        return vec![0];
    }
    let mut data: Vec<_> = symbols
        .iter()
        .filter_map(|symbol| match symbol.get_section() {
            symbols::Section::Data => Some((symbol.get_offset(), symbol.get_data())),
            _ => None,
        })
        .collect();
    data.sort_by_key(|(offset, _)| *offset);
    data.iter().fold(vec![], |mut acc, (_, ref d)| {
        acc.extend(*d);
        acc
    })
}

pub fn build_relocation_section(
    section_layout: &mut SectionLayout,
    relocation_information: HashMap<String, Vec<RelocationInformation>>,
) -> Vec<u8> {
    let size_of_relocation = mem::size_of::<RelocationInformation>() as u32;
    let mut relocations_end = section_layout.get_sections_end();

    let mut relocations = vec![];
    for (_, section) in section_layout.sections.iter_mut() {
        let relocation_information = match relocation_information.get(&section.name) {
            Some(r) => r,
            None => continue,
        };

        section.pointer_to_relocations = relocations_end;
        section.number_of_relocations = relocation_information.len() as u16;
        for relocation in relocation_information {
            relocations.extend(relocation.virtual_address.to_le_bytes());
            relocations.extend(relocation.symbol_table_index.to_le_bytes());
            relocations.extend(relocation._type.to_le_bytes());
            relocations_end += size_of_relocation;
        }
    }

    relocations
}
