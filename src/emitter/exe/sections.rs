use std::{
    collections::{BTreeMap, HashMap},
    mem, usize, vec,
};

use crate::emitter::{elf::sections, structs::Sliceable};

use super::defs::*;

pub const BASE_OF_CODE: u32 = 0x1000;
pub const FILE_ALIGNMENT: u32 = 0x200;
pub const SECTION_ALIGNMENT: u32 = 0x1000;

pub fn round_to_multiple(num: u32, multiple: u32) -> u32 {
    multiple * ((num - 1) / multiple) + multiple
}

const DOS_STUB: [u8; 56] = [
    0x0E, 0x1F, 0xBA, 0x0E, 0x00, 0xB4, 0x09, 0xCD, 0x21, 0xB8, 0x01, 0x4C, 0xCD, 0x21, 0x54, 0x68,
    0x69, 0x73, 0x20, 0x70, 0x72, 0x6F, 0x67, 0x72, 0x61, 0x6D, 0x20, 0x63, 0x61, 0x6E, 0x6E, 0x6F,
    0x74, 0x20, 0x62, 0x65, 0x20, 0x72, 0x75, 0x6E, 0x20, 0x69, 0x6E, 0x20, 0x44, 0x4F, 0x53, 0x20,
    0x6D, 0x6F, 0x64, 0x65, 0x2E, 0x24, 0x00, 0x00,
];

const IMAGE_NUMBEROF_DIRECTORY_ENTRIES: usize = 16;
const IMAGE_SIZEOF_SHORT_NAME: usize = 8;

const IMAGE_BASE: u64 = 0x140000000;

#[derive(Debug, Clone)]
pub enum SectionData {
    Data(Vec<u8>),
    DataCallback(fn(u32) -> Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct Section {
    name: String,
    data: SectionData,
    virtual_address: u32,
    virtual_size: u32,
    pointer_to_raw_data: u32,
    size_of_raw_data: u32,
    characteristics: u32,
}

impl Section {
    pub fn new(name: String, data: SectionData, characteristics: u32) -> Self {
        Self {
            name,
            data,
            characteristics,
            virtual_address: 0,
            virtual_size: 0,
            pointer_to_raw_data: 0,
            size_of_raw_data: 0,
        }
    }

    pub fn get_header(&self) -> Vec<u8> {
        match &self.data {
            SectionData::DataCallback(_) => {
                panic!("Cannot produce section header for section with data callback")
            }
            SectionData::Data(_) => (),
        };
        build_section_header(
            &self.name,
            self.virtual_address,
            self.virtual_size,
            self.pointer_to_raw_data,
            self.size_of_raw_data,
            self.characteristics,
        )
    }

    fn get_data(&self) -> Vec<u8> {
        match &self.data {
            SectionData::Data(data) => data.to_vec(),
            SectionData::DataCallback(data) => data(self.virtual_address),
        }
    }

    pub fn get_data_aligned(&self) -> Vec<u8> {
        let mut data = self.get_data().to_vec();
        let diff = FILE_ALIGNMENT as usize - data.len();
        data.extend(std::iter::repeat(0x0).take(diff));
        data
    }
}

#[derive(Debug)]
pub struct PELayout {
    sections: BTreeMap<String, Section>,
    size_of_headers: u32,
}

impl PELayout {
    pub fn new(sections: Vec<Section>) -> Self {
        let size_of_dos_header = mem::size_of::<DOSHeader>() as u32;
        let size_of_optional_header = mem::size_of::<OptionalHeader>() as u32;
        let size_of_section_header = sections.len() as u32 * mem::size_of::<SectionHeader>() as u32;

        let size_of_headers = size_of_dos_header + size_of_optional_header + size_of_section_header;
        let size_of_headers = round_to_multiple(size_of_headers, FILE_ALIGNMENT);

        let mut sections_end = size_of_headers;
        let mut sections_end_virtual = BASE_OF_CODE;

        let mut _sections = BTreeMap::new();
        for mut section in sections {
            section.pointer_to_raw_data = sections_end;
            section.virtual_address = sections_end_virtual;

            let data = section.get_data();
            let aligned_data = section.get_data_aligned();
            let virtual_size = data.len() as u32;
            let size_of_raw_data = aligned_data.len() as u32;
            section.data = SectionData::Data(data);
            println!("{size_of_raw_data}, {virtual_size}");
            section.size_of_raw_data = size_of_raw_data;
            section.virtual_size = virtual_size;

            sections_end =
                round_to_multiple(sections_end + section.size_of_raw_data, FILE_ALIGNMENT);
            sections_end_virtual = round_to_multiple(
                sections_end_virtual + section.virtual_size,
                SECTION_ALIGNMENT,
            );
            _sections.insert(section.name.clone(), section);
        }
        // println!("{:#?}", _sections);
        Self {
            sections: _sections,
            size_of_headers,
        }
    }

    pub fn get_section(&self, name: &str) -> Section {
        self.sections.get(name).unwrap().clone()
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct DOSHeader {
    e_magic: u16,      // Magic number
    e_cblp: u16,       // Bytes on last page of file
    e_cp: u16,         // Pages in file
    e_crlc: u16,       // Relocations
    e_cparhdr: u16,    // Size of header in paragraphs
    e_minalloc: u16,   // Minimum extra paragraphs needed
    e_maxalloc: u16,   // Maximum extra paragraphs needed
    e_ss: u16,         // Initial (relative) SS value
    e_sp: u16,         // Initial SP value
    e_csum: u16,       // Checksum
    e_ip: u16,         // Initial IP value
    e_cs: u16,         // Initial (relative) CS value
    e_lfarlc: u16,     // File address of relocation table
    e_ovno: u16,       // Overlay number
    e_res: [u16; 4],   // Reserved s
    e_oemid: u16,      // OEM identifier (for e_oeminfo)
    e_oeminfo: u16,    // OEM information: u16, e_oemid specific
    e_res2: [u16; 10], // Reserved s
    e_lfanew: u32,     // File address of new exe header
    dos_stub: [u8; 56],
}

impl Sliceable for DOSHeader {}

#[derive(Debug)]
#[repr(C)]
pub struct NTHeaders {
    pe_signature: [u8; 4],
    file_header: FileHeader,
    optional_header: OptionalHeader,
}

impl Sliceable for NTHeaders {}

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
pub struct ImageDataDirectories {
    export_directory: u64,
    import_directory: ImageDataDirectory,
    stub0: [u64; 3],
    relocation_table: ImageDataDirectory,
    stub1: [u64; 6],
    import_address_table: ImageDataDirectory,
    stub2: [u64; 2],
    reserverd: u32,
}

impl Sliceable for ImageDataDirectories {}

#[derive(Debug)]
#[repr(C)]
pub struct OptionalHeader {
    magic: u16,
    major_linker_version: u8,
    minor_linker_version: u8,
    size_of_code: u32,
    size_of_initialized_data: u32,
    size_of_uninitialized_data: u32,
    address_of_entry_point: u32,
    base_of_code: u32,
    image_base: u64,
    section_alignment: u32,
    file_alignment: u32,
    major_operating_system_version: u16,
    minor_operating_system_version: u16,
    major_image_version: u16,
    minor_image_version: u16,
    major_subsystem_version: u16,
    minor_subsystem_version: u16,
    win32_version_value: u32,
    size_of_image: u32,
    size_of_headers: u32,
    check_sum: u32,
    subsystem: u16,
    dll_characteristics: u16,
    size_of_stack_reserve: u64,
    size_of_stack_commit: u64,
    size_of_heap_reserve: u64,
    size_of_heap_commit: u64,
    loader_flags: u32,
    number_of_rva_and_sizes: u32,
    data_directory: ImageDataDirectories,
}

impl Sliceable for OptionalHeader {}

#[derive(Debug)]
#[repr(C)]
pub struct ImageDataDirectory {
    virtual_address: u32,
    size: u32,
}

impl Sliceable for ImageDataDirectory {}

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

#[derive(Debug)]
#[repr(C)]
pub struct ImageImportDescriptor {
    original_first_thunk: u32,
    time_date_stamp: u32,
    forwarder_chain: u32,
    name: u32,
    first_thunk: u32,
}

impl Sliceable for ImageImportDescriptor {}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ImageImportByName {
    hint: u16,
    name: u8,
}

impl Sliceable for ImageImportByName {}

#[derive(Debug)]
#[repr(C)]
pub struct ImageBaseRelocation {
    virtual_address: u32,
    size_of_block: u32,
}

impl Sliceable for ImageBaseRelocation {}

pub fn build_dos_header() -> Vec<u8> {
    let e_lfanew = mem::size_of::<DOSHeader>() as u32;
    DOSHeader {
        e_magic: 0x5A4D,
        e_cblp: 0,
        e_cp: 0,
        e_crlc: 0,
        e_cparhdr: 0,
        e_minalloc: 0,
        e_maxalloc: 0,
        e_ss: 0,
        e_sp: 0,
        e_csum: 0,
        e_ip: 0,
        e_cs: 0,
        e_lfarlc: 0,
        e_ovno: 0,
        e_res: [0; 4],
        e_oemid: 0,
        e_oeminfo: 0,
        e_res2: [0; 10],
        e_lfanew,
        dos_stub: DOS_STUB,
    }
    .as_vec()
}

pub fn build_nt_header(
    created_at: u32,
    number_of_sections: u16,
    size_of_optional_header: u32,
    size_of_headers: u32,
    text_size: u32,
    data_size: u32,
    bss_size: u32,
) -> Vec<u8> {
    NTHeaders {
        pe_signature: [b'P', b'E', 0, 0],
        file_header: FileHeader {
            machine: IMAGE_FILE_MACHINE_AMD64,
            number_of_sections,
            time_date_stamp: created_at,
            pointer_to_symbol_table: 0,
            number_of_symbols: 0,
            size_of_optional_header: size_of_optional_header as u16,
            characteristics: IMAGE_FILE_EXECUTABLE_IMAGE | IMAGE_FILE_LARGE_ADDRESS_AWARE,
        },
        optional_header: OptionalHeader {
            magic: IMAGE_FILE_MACHINE_TYPR_PE32_PLUS,
            major_linker_version: 0xe,
            minor_linker_version: 0x0,
            size_of_code: text_size,
            size_of_initialized_data: data_size,
            size_of_uninitialized_data: bss_size,
            address_of_entry_point: BASE_OF_CODE,
            base_of_code: BASE_OF_CODE,
            image_base: IMAGE_BASE,
            section_alignment: SECTION_ALIGNMENT,
            file_alignment: FILE_ALIGNMENT,
            major_operating_system_version: 0x6,
            minor_operating_system_version: 0x0,
            major_image_version: 0x0,
            minor_image_version: 0x0,
            major_subsystem_version: 0x6,
            minor_subsystem_version: 0x0,
            win32_version_value: 0x0,
            size_of_image: 0x5000,
            size_of_headers,
            check_sum: 0x0,
            subsystem: IMAGE_SUBSYSTEM_WINDOWS_CUI,
            dll_characteristics: IMAGE_DLLCHARACTERISTICS_HIGH_ENTROPY_VA
                | IMAGE_DLLCHARACTERISTICS_DYNAMIC_BASE
                | IMAGE_DLLCHARACTERISTICS_NX_COMPAT
                | IMAGE_DLLCHARACTERISTICS_TERMINAL_SERVER_AWARE,
            size_of_stack_reserve: 0x100000,
            size_of_stack_commit: 0x1000,
            size_of_heap_reserve: 0x100000,
            size_of_heap_commit: 0x1000,
            loader_flags: 0x0,
            number_of_rva_and_sizes: 0x10,
            data_directory: build_data_directories(),
        },
    }
    .as_vec()
}

fn build_data_directories() -> ImageDataDirectories {
    ImageDataDirectories {
        export_directory: 0,
        import_directory: ImageDataDirectory {
            virtual_address: 0x2000,
            size: 0x3c, //TODO: calculate programmaticallly
        },
        stub0: [0; 3],
        relocation_table: ImageDataDirectory {
            virtual_address: 0x4000,
            size: 0xC, //TODO: calculate programmaticallly
        },
        stub1: [0; 6],
        import_address_table: ImageDataDirectory {
            virtual_address: 0x2064, //TODO: calculate programmaticallly
            size: 0x28,              //TODO: calculate programmaticallly
        },
        stub2: [0; 2],
        reserverd: 0,
    }
}

pub fn build_section_header(
    name: &str,
    virtual_address: u32,
    virtual_size: u32,
    pointer_to_raw_data: u32,
    size_of_raw_data: u32,
    characteristics: u32,
) -> Vec<u8> {
    let name_bytes = name.as_bytes();
    let mut name = [0; 8];
    name[..name_bytes.len()].copy_from_slice(name_bytes);
    SectionHeader {
        name,
        virtual_size,
        virtual_address,
        size_of_raw_data,
        pointer_to_raw_data,
        pointer_to_relocations: 0x0,
        pointer_to_linenumbers: 0x0,
        number_of_relocations: 0x0,
        number_of_linenumbers: 0x0,
        characteristics,
    }
    .as_vec()
}

pub fn build_import_directory(offset: u32) -> Vec<u8> {
    let lookup_table_offset = offset + 3 * mem::size_of::<ImageImportDescriptor>() as u32;

    println!("{lookup_table_offset:0x}");
    let lookup_table_size = 5 * mem::size_of::<u64>() as u32;
    let hint_table_offset = lookup_table_offset + 2 * lookup_table_size;
    println!("{hint_table_offset:0x}");

    let hint_names = [
        "ExitProcess\0".as_bytes(),
        "__acrt_iob_func\0".as_bytes(),
        "__stdio_common_vfprintf\0".as_bytes(),
    ];

    let lookup_table = [
        (hint_table_offset as u64).to_le_bytes(),
        0_u64.to_le_bytes(),
        (hint_table_offset as u64 + (mem::size_of::<u16>() + hint_names[0].len()) as u64)
            .to_le_bytes(),
        (hint_table_offset as u64
            + (mem::size_of::<u16>()
                + hint_names[0].len()
                + mem::size_of::<u16>()
                + hint_names[1].len()) as u64)
            .to_le_bytes(),
        0_u64.to_le_bytes(),
    ]
    .concat();
    let address_table = lookup_table.clone();
    let address_table_offset = lookup_table_offset + lookup_table_size;

    let hint_name_table: Vec<u8> = [
        [&0x167_u16.to_le_bytes(), hint_names[0]].concat(),
        [&0_u16.to_le_bytes(), hint_names[1]].concat(),
        [&0x3_u16.to_le_bytes(), hint_names[2]].concat(),
        [&0x3_u16.to_le_bytes(), hint_names[2]].concat(),
    ]
    .concat();

    let names_offset = hint_table_offset + hint_name_table.len() as u32;
    let names = [
        "KERNEL32.dll\0".as_bytes(),
        "api-ms-win-crt-stdio-l1-1-0.dll\0".as_bytes(),
    ];

    let directory_table = [
        ImageImportDescriptor {
            original_first_thunk: lookup_table_offset,
            time_date_stamp: 0,
            forwarder_chain: 0,
            name: names_offset,
            first_thunk: address_table_offset,
        }
        .as_vec(),
        ImageImportDescriptor {
            original_first_thunk: lookup_table_offset + 2 * mem::size_of::<u64>() as u32,
            time_date_stamp: 0,
            forwarder_chain: 0,
            name: names_offset + names[0].len() as u32,
            first_thunk: address_table_offset + 2 * mem::size_of::<u64>() as u32,
        }
        .as_vec(),
        ImageImportDescriptor {
            original_first_thunk: 0,
            time_date_stamp: 0,
            forwarder_chain: 0,
            name: 0,
            first_thunk: 0,
        }
        .as_vec(),
    ]
    .concat();
    [
        directory_table,
        lookup_table,
        address_table,
        hint_name_table,
        names.concat(),
    ]
    .concat()
}

pub fn build_relocation_section() -> Vec<u8> {
    let number_of_relocations = 2;
    let size_of_block = mem::size_of::<ImageBaseRelocation>() as u32 + 2 * number_of_relocations;
    [
        ImageBaseRelocation {
            virtual_address: BASE_OF_CODE,
            size_of_block,
        }
        .as_vec(),
        (((0x1F) | 0xA0 << 8) as u16).to_le_bytes().to_vec(),
    ]
    .concat()
}
