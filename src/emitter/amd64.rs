use crate::emitter::elf::structs::Sliceable;

pub const STDOUT_FD: i32 = 0x1;
pub const SYS_WRITE: i32 = 0x4;

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Register {
    Eax = 0x0,
    Ecx = 0x1,
    Edx = 0x2,
    Ebx = 0x3,
    Esp = 0x4,
    Ebp = 0x5,
    Esi = 0x6,
    Edi = 0x7,
}

impl From<Register> for u32 {
    fn from(reg: Register) -> Self {
        reg as u32
    }
}

#[allow(dead_code)]
#[repr(packed)]
pub struct Mov32 {
    opcode: u8,
    value: i32,
}

impl Mov32 {
    pub fn build(reg: Register, value: i32) -> Vec<u8> {
        Mov32 {
            opcode: 0xb8 + reg as u8,
            value,
        }
        .as_vec()
    }
}

impl Sliceable for Mov32 {}

#[allow(dead_code)]
#[repr(packed)]
pub struct Mov32rr {
    opcode: u8,
    mod_rm: u8,
}

impl Mov32rr {
    pub fn build(dst: Register, src: Register) -> Vec<u8> {
        Mov32rr {
            opcode: 0x89,
            mod_rm: 3 << 6 | (src as u8) << 3 | dst as u8,
        }
        .as_vec()
    }
}

impl Sliceable for Mov32rr {}

#[allow(dead_code)]
#[repr(packed)]
pub struct Sub32 {
    opcode: u8,
    mod_rm: u8,
    value: i32,
}

impl Sub32 {
    pub fn build(reg: Register, value: i32) -> Vec<u8> {
        Sub32 {
            opcode: 0x81,
            mod_rm: 3 << 6 | 5 << 3 | (reg as u8),
            value,
        }
        .as_vec()
    }
}

impl Sliceable for Sub32 {}

#[allow(dead_code)]
#[repr(packed)]
pub struct SysCall {
    opcode: u16,
}

impl SysCall {
    pub fn build() -> Vec<u8> {
        SysCall { opcode: 0x80CD }.as_vec()
    }
}

impl Sliceable for SysCall {}

#[allow(dead_code)]
#[repr(packed)]
pub struct Push32 {
    opcode: u8,
    value: i32,
}

impl Push32 {
    pub fn build(value: i32) -> Vec<u8> {
        Self {
            opcode: 0x68,
            value,
        }
        .as_vec()
    }
}

impl Sliceable for Push32 {}

#[allow(dead_code)]
#[repr(packed)]
pub struct Push32rr {
    opcode: u8,
}

impl Push32rr {
    pub fn build(reg: Register) -> Vec<u8> {
        Self {
            opcode: 0x50 + reg as u8,
        }
        .as_vec()
    }
}

impl Sliceable for Push32rr {}

#[allow(dead_code)]
#[repr(packed)]
pub struct Pop32rr {
    opcode: u8,
}

impl Pop32rr {
    pub fn build(reg: Register) -> Vec<u8> {
        Self {
            opcode: 0x58 + reg as u8,
        }
        .as_vec()
    }
}

impl Sliceable for Pop32rr {}
