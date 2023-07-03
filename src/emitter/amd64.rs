use crate::emitter::elf::structs::Sliceable;

pub const STDOUT_FD: i32 = 0x1;

pub const SYS_WRITE: i32 = 0x1;
pub const SYS_EXIT: i32 = 0x3c;

const REX_WRITE: u8 = 0b01001000;
const REX_READ: u8 = 0b01000100;
const REX_X: u8 = 0b01000010;
const REX_B: u8 = 0b01000001;

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Register {
    Ax = 0x0,
    Cx = 0x1,
    Dx = 0x2,
    Bx = 0x3,
    Sp = 0x4,
    Bp = 0x5,
    Si = 0x6,
    Di = 0x7,
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
        SysCall { opcode: 0x050f }.as_vec()
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
pub struct Push {
    opcode: u8,
}

impl Push {
    pub fn build(reg: Register) -> Vec<u8> {
        Self {
            opcode: 0x50 + reg as u8,
        }
        .as_vec()
    }
}

impl Sliceable for Push {}

#[allow(dead_code)]
#[repr(packed)]
pub struct Pop {
    opcode: u8,
}

impl Pop {
    pub fn build(reg: Register) -> Vec<u8> {
        Self {
            opcode: 0x58 + reg as u8,
        }
        .as_vec()
    }
}

impl Sliceable for Pop {}

#[allow(dead_code)]
#[repr(packed)]
pub struct Mov64 {
    rex: u8,
    opcode: u8,
    value: i64,
}

impl Mov64 {
    pub fn build(reg: Register, value: i64) -> Vec<u8> {
        Self {
            rex: REX_WRITE,
            opcode: 0xb8 + reg as u8,
            value,
        }
        .as_vec()
    }
}

impl Sliceable for Mov64 {}

#[allow(dead_code)]
#[repr(packed)]
pub struct Mov64rr {
    rex: u8,
    opcode: u8,
    mod_rm: u8,
}

impl Mov64rr {
    pub fn build(dst: Register, src: Register) -> Vec<u8> {
        Self {
            rex: REX_WRITE,
            opcode: 0x89,
            mod_rm: 3 << 6 | (src as u8) << 3 | dst as u8,
        }
        .as_vec()
    }
}

impl Sliceable for Mov64rr {}

#[allow(dead_code)]
#[repr(packed)]
pub struct Sub64 {
    rex: u8,
    opcode: u8,
    mod_rm: u8,
    value: i32,
}

impl Sub64 {
    pub fn build(reg: Register, value: i32) -> Vec<u8> {
        Self {
            rex: REX_WRITE,
            opcode: 0x81,
            mod_rm: 3 << 6 | 5 << 3 | (reg as u8),
            value,
        }
        .as_vec()
    }
}

impl Sliceable for Sub64 {}
