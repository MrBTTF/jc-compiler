pub const ELF_MAGIC: [u8; 4] = [0x7F, 0x45, 0x4c, 0x46];

pub const ELF_CLASS_NONE: u8 = 0x0;
pub const ELF_CLASS_32_BIT: u8 = 0x1;
pub const ELF_CLASS_64_BIT: u8 = 0x2;

pub const ELF_DATA_NONE: u8 = 0x0;
pub const ELF_DATA_LITTLE_ENDIAN: u8 = 0x1;
pub const ELF_DATA_BIG_ENDIAN: u8 = 0x2;

pub const ELF_VERSION_NONE: u8 = 0;
pub const ELF_VERSION_CURRENT: u8 = 1;
pub const ELF_VERSION_NUM: u8 = 2;

pub const ELF_OSABI_SYSTEM_V: u8 = 0;
pub const ELF_OSABI_LINUX: u8 = 3;

pub const ELF_ABI_VERSION_NONE: u8 = 0;

pub const ELF_TYPE_NONE: u16 = 0;
pub const ELF_TYPE_REL: u16 = 1;
pub const ELF_TYPE_EXEC: u16 = 2;
pub const ELF_TYPE_DYN: u16 = 3;
pub const ELF_TYPE_CORE: u16 = 4;
pub const ELF_TYPE_LOPROC: u16 = 0xff00;
pub const ELF_TYPE_HIPROC: u16 = 0xffff;

pub const ELF_MACHINE_NONE: u16 = 0;
pub const ELF_MACHINE_M32: u16 = 1;
pub const ELF_MACHINE_SPARC: u16 = 2;
pub const ELF_MACHINE_386: u16 = 3;
pub const ELF_MACHINE_68K: u16 = 4;
pub const ELF_MACHINE_88K: u16 = 5;
pub const ELF_MACHINE_486: u16 = 6; /* Not used in Linux at least */
pub const ELF_MACHINE_860: u16 = 7;
pub const ELF_MACHINE_MIPS: u16 = 8; /* R3k, bigendian(?) */
pub const ELF_MACHINE_MIPS_RS4_BE: u16 = 10; /* R4k BE */
pub const ELF_MACHINE_PARISC: u16 = 15;
pub const ELF_MACHINE_SPARC32PLUS: u16 = 18;
pub const ELF_MACHINE_PPC: u16 = 20;
pub const ELF_MACHINE_PPC64: u16 = 21;
pub const ELF_MACHINE_S390: u16 = 22;
pub const ELF_MACHINE_SH: u16 = 42;
pub const ELF_MACHINE_SPARCV9: u16 = 43; /* v9 = SPARC64 */
pub const ELF_MACHINE_H8_300H: u16 = 47;
pub const ELF_MACHINE_H8S: u16 = 48;
pub const ELF_MACHINE_IA_64: u16 = 50;
pub const ELF_MACHINE_X86_64: u16 = 62;
pub const ELF_MACHINE_CRIS: u16 = 76;
pub const ELF_MACHINE_V850: u16 = 87;
pub const ELF_MACHINE_ALPHA: u16 = 0x9026; /* Interim Alpha that stuck around */
pub const ELF_MACHINE_CYGNUS_V850: u16 = 0x9080; /* Old v850 ID used by Cygnus */
pub const ELF_MACHINE_S390_OLD: u16 = 0xA390; /* Obsolete interim value for S/390 */

pub const PROGRAM_TYPE_NULL: u32 = 0;
pub const PROGRAM_TYPE_LOAD: u32 = 1;
pub const PROGRAM_TYPE_DYNAMIC: u32 = 2;
pub const PROGRAM_TYPE_INTERP: u32 = 3;
pub const PROGRAM_TYPE_NOTE: u32 = 4;
pub const PROGRAM_TYPE_SHLIB: u32 = 5;
pub const PROGRAM_TYPE_PHDR: u32 = 6;
pub const PROGRAM_TYPE_LOOS: u32 = 0x60000000;
pub const PROGRAM_TYPE_HIOS: u32 = 0x6fffffff;
pub const PROGRAM_TYPE_LOPROC: u32 = 0x70000000;
pub const PROGRAM_TYPE_HIPROC: u32 = 0x7fffffff;
pub const PROGRAM_TYPE_GNU_EH_FRAME: u32 = 0x6474e550; /* Extension, eh? */

pub const PROGRAM_HEADER_EXEC: u32 = 0x1;
pub const PROGRAM_HEADER_WRITE: u32 = 0x2;
pub const PROGRAM_HEADER_READ: u32 = 0x4;

pub const SEGMENT_TYPE_NULL: u32 = 0;
pub const SEGMENT_TYPE_PROGBITS: u32 = 1;
pub const SEGMENT_TYPE_SYMTAB: u32 = 2;
pub const SEGMENT_TYPE_STRTAB: u32 = 3;
pub const SEGMENT_TYPE_RELA: u32 = 4;
pub const SEGMENT_TYPE_HASH: u32 = 5;
pub const SEGMENT_TYPE_DYNAMIC: u32 = 6;
pub const SEGMENT_TYPE_NOTE: u32 = 7;
pub const SEGMENT_TYPE_NOBITS: u32 = 8;
pub const SEGMENT_TYPE_REL: u32 = 9;
pub const SEGMENT_TYPE_SHLIB: u32 = 10;
pub const SEGMENT_TYPE_DYNSYM: u32 = 11;
pub const SEGMENT_TYPE_INIT_ARRAY: u32 = 14;
pub const SEGMENT_TYPE_FINI_ARRAY: u32 = 15;
pub const SEGMENT_TYPE_PREINIT_ARRAY: u32 = 16;
pub const SEGMENT_TYPE_GROUP: u32 = 17;
pub const SEGMENT_TYPE_SYMTAB_SHNDX: u32 = 18;
pub const SEGMENT_TYPE_LOPROC: u32 = 0x70000000;
pub const SEGMENT_TYPE_HIPROC: u32 = 0x7fffffff;
pub const SEGMENT_TYPE_LOUSER: u32 = 0x80000000;
pub const SEGMENT_TYPE_HIUSER: u32 = 0xffffffff;

pub const SEGMENT_FLAGS_NONE: u32 = 0; /* Writable */
pub const SEGMENT_FLAGS_WRITE: u32 = 1 << 0; /* Writable */
pub const SEGMENT_FLAGS_ALLOC: u32 = 1 << 1; /* Occupies memory during execution */
pub const SEGMENT_FLAGS_EXECINSTR: u32 = 1 << 2; /* Executable */
pub const SEGMENT_FLAGS_MERGE: u32 = 1 << 4; /* Might be merged */
pub const SEGMENT_FLAGS_STRINGS: u32 = 1 << 5; /* Contains nul-terminated strings */
pub const SEGMENT_FLAGS_INFO_LINK: u32 = 1 << 6; /* `sh_info' contains SHT index */
pub const SEGMENT_FLAGS_LINK_ORDER: u32 = 1 << 7; /* Preserve order after combining */
pub const SEGMENT_FLAGS_OS_NONCONFORMING: u32 = 1 << 8; /* Non-standard OS specific handling required */
pub const SEGMENT_FLAGS_GROUP: u32 = 1 << 9; /* Section is member of a group */
pub const SEGMENT_FLAGS_TLS: u32 = 1 << 10; /* Section hold thread-local data */
