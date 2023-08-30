use std::mem;

use crate::emitter::structs::Sliceable;

use super::defs::*;

const DOS_STUB: [u8; 64] = [
    0x0E, 0x1F, 0xBA, 0x0E, 0x00, 0xB4, 0x09, 0xCD, 0x21, 0xB8, 0x01, 0x4C, 0xCD, 0x21, 0x54, 0x68,
    0x69, 0x73, 0x20, 0x70, 0x72, 0x6F, 0x67, 0x72, 0x61, 0x6D, 0x20, 0x63, 0x61, 0x6E, 0x6E, 0x6F,
    0x74, 0x20, 0x62, 0x65, 0x20, 0x72, 0x75, 0x6E, 0x20, 0x69, 0x6E, 0x20, 0x44, 0x4F, 0x53, 0x20,
    0x6D, 0x6F, 0x64, 0x65, 0x2E, 0x0D, 0x0D, 0x0A, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

const IMAGE_NUMBEROF_DIRECTORY_ENTRIES: usize = 16;
const IMAGE_SIZEOF_SHORT_NAME: usize = 8;

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
    dos_stub: [u8; 64],
}

impl Sliceable for DOSHeader {}

#[derive(Debug)]
#[repr(C)]
pub struct NTHeaders {
    pe_signature: u32,
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
    data_irectory: [ImageDataDirectory; IMAGE_NUMBEROF_DIRECTORY_ENTRIES],
}

impl Sliceable for OptionalHeader {}

#[derive(Debug)]
#[repr(C)]
pub struct ImageDataDirectory {
    virtual_address: u16,
    size: u16,
}

impl Sliceable for ImageDataDirectory {}

#[derive(Debug)]
#[repr(C)]
pub struct SectionTable {
    pe_signature: [u8; 4],
}

impl Sliceable for SectionTable {}

#[derive(Debug)]
#[repr(C)]
pub struct ExeHeader {
    dos_header: DOSHeader,
    nt_header: NTHeaders,
}

impl Sliceable for ExeHeader {}

#[derive(Debug)]
#[repr(C)]
pub struct SectionHeader {
    name: [u8; IMAGE_SIZEOF_SHORT_NAME],
    misc: u16,
    virtual_address: u16,
    size_of_raw_data: u16,
    pointer_to_raw_data: u16,
    pointer_to_relocations: u16,
    pointer_to_linenumbers: u16,
    number_of_relocations: u8,
    number_of_linenumbers: u8,
    characteristics: u16,
}

impl Sliceable for SectionHeader {}

fn build_header(created_at: u32) -> ExeHeader {
    ExeHeader {
        dos_header: build_dos_header(),
        nt_header: build_nt_header(created_at),
    }
}

fn build_dos_header() -> DOSHeader {
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
}

fn build_nt_header(created_at: u32) -> NTHeaders {
    let size_of_optional_header = mem::size_of::<OptionalHeader>() as u16;
    NTHeaders {
        pe_signature: 0x50450000,
        file_header: FileHeader {
            machine: IMAGE_FILE_MACHINE_I386,
            number_of_sections: todo!(),
            time_date_stamp: created_at,
            pointer_to_symbol_table: 0,
            number_of_symbols: 0,
            size_of_optional_header,
            characteristics: IMAGE_FILE_EXECUTABLE_IMAGE | IMAGE_FILE_LARGE_ADDRESS_AWARE,
        },
        optional_header: OptionalHeader {
            magic: IMAGE_FILE_MACHINE_PE32_PLUS,
            major_linker_version: todo!(),
            minor_linker_version: todo!(),
            size_of_code: todo!(),
            size_of_initialized_data: todo!(),
            size_of_uninitialized_data: todo!(),
            address_of_entry_point: todo!(),
            base_of_code: todo!(),
            image_base: todo!(),
            section_alignment: todo!(),
            file_alignment: todo!(),
            major_operating_system_version: todo!(),
            minor_operating_system_version: todo!(),
            major_image_version: todo!(),
            minor_image_version: todo!(),
            major_subsystem_version: todo!(),
            minor_subsystem_version: todo!(),
            win32_version_value: todo!(),
            size_of_image: todo!(),
            size_of_headers: todo!(),
            check_sum: todo!(),
            subsystem: todo!(),
            dll_characteristics: todo!(),
            size_of_stack_reserve: todo!(),
            size_of_stack_commit: todo!(),
            size_of_heap_reserve: todo!(),
            size_of_heap_commit: todo!(),
            loader_flags: todo!(),
            number_of_rva_and_sizes: todo!(),
            data_irectory: todo!(),
        },
    }
}
