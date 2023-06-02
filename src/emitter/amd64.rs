use crate::emitter::elf::structs::Sliceable;

pub const STDOUT_FD: i32 = 0x1;
pub const SYS_WRITE: i32 = 0x4;

#[repr(u8)]
pub enum Register {
    Eax = 0x0,
    Ecx = 0x1,
    Edx = 0x2,
    Ebx = 0x3,
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
pub struct SysCall {
    opcode: u16,
}

impl SysCall {
    pub fn build() -> Vec<u8> {
        SysCall { opcode: 0x80CD }.as_vec()
    }
}

impl Sliceable for SysCall {}
