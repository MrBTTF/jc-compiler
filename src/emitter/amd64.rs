

use super::elf::structs::Instruction;

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
    pub fn new(reg: Register, value: i32) -> Box<Self> {
        Box::new(Self {
            opcode: 0xb8 + reg as u8,
            value,
        })
    }
}

impl Instruction for Mov32 {}

#[allow(dead_code)]
#[repr(packed)]
pub struct Mov32rr {
    opcode: u8,
    mod_rm: u8,
}

impl Mov32rr {
    pub fn new(dst: Register, src: Register) -> Box<Self> {
        Box::new(Self {
            opcode: 0x89,
            mod_rm: 3 << 6 | (src as u8) << 3 | dst as u8,
        })
    }
}

impl Instruction for Mov32rr {}

#[allow(dead_code)]
#[repr(packed)]
pub struct Sub32 {
    opcode: u8,
    mod_rm: u8,
    value: i32,
}

impl Sub32 {
    pub fn new(reg: Register, value: i32) -> Box<Self> {
        Box::new(Self {
            opcode: 0x81,
            mod_rm: 3 << 6 | 5 << 3 | (reg as u8),
            value,
        })
    }
}

impl Instruction for Sub32 {}

#[allow(dead_code)]
#[repr(packed)]
pub struct Add32 {
    opcode: u8,
    mod_rm: u8,
    value: i32,
}

impl Add32 {
    pub fn new(reg: Register, value: i32) -> Box<Self> {
        Box::new(Self {
            opcode: 0x81,
            mod_rm: 3 << 6 | (reg as u8),
            value,
        })
    }
}

impl Instruction for Add32 {}

#[allow(dead_code)]
#[repr(packed)]
pub struct Shl32 {
    opcode: u8,
    mod_rm: u8,
    value: u8,
}

impl Shl32 {
    pub fn new(reg: Register, value: u8) -> Box<Self> {
        Box::new(Self {
            opcode: 0xc1,
            mod_rm: 3 << 6 | 4 << 3 | (reg as u8),
            value,
        })
    }
}

impl Instruction for Shl32 {}

#[allow(dead_code)]
#[repr(packed)]
pub struct Xor32rr {
    opcode: u8,
    mod_rm: u8,
}

impl Xor32rr {
    pub fn new(dst: Register, src: Register) -> Box<Self> {
        Box::new(Self {
            opcode: 0x31,
            mod_rm: 3 << 6 | (src as u8) << 3 | dst as u8,
        })
    }
}

impl Instruction for Xor32rr {}

#[allow(dead_code)]
#[repr(packed)]
pub struct Div32 {
    opcode: u8,
    mod_rm: u8,
}

impl Div32 {
    pub fn new(divider: Register) -> Box<Self> {
        Box::new(Self {
            opcode: 0xf7,
            mod_rm: 3 << 6 | 6 << 3 | divider as u8,
        })
    }
}

impl Instruction for Div32 {}

#[allow(dead_code)]
#[repr(packed)]
pub struct Or32rr {
    opcode: u8,
    mod_rm: u8,
}

impl Or32rr {
    pub fn new(dst: Register, src: Register) -> Box<Self> {
        Box::new(Self {
            opcode: 0x9,
            mod_rm: 3 << 6 | (src as u8) << 3 | dst as u8,
        })
    }
}

impl Instruction for Or32rr {}

#[allow(dead_code)]
#[repr(packed)]
pub struct SysCall {
    opcode: u16,
}

impl SysCall {
    pub fn new() -> Box<Self> {
        Box::new(SysCall { opcode: 0x050f })
    }
}

impl Instruction for SysCall {}

#[allow(dead_code)]
#[repr(packed)]
pub struct Push32 {
    opcode: u8,
    value: i32,
}

impl Push32 {
    pub fn new(value: i32) -> Box<Self> {
        Box::new(Self {
            opcode: 0x68,
            value,
        })
    }
}

impl Instruction for Push32 {}

#[allow(dead_code)]
#[repr(packed)]
pub struct Push {
    opcode: u8,
}

impl Push {
    pub fn new(reg: Register) -> Box<Self> {
        Box::new(Self {
            opcode: 0x50 + reg as u8,
        })
    }
}

impl Instruction for Push {}

#[allow(dead_code)]
#[repr(packed)]
pub struct Pop {
    opcode: u8,
}

impl Pop {
    pub fn new(reg: Register) -> Box<Self> {
        Box::new(Self {
            opcode: 0x58 + reg as u8,
        })
    }
}

impl Instruction for Pop {}

#[allow(dead_code)]
#[repr(packed)]
pub struct Mov64 {
    rex: u8,
    opcode: u8,
    value: i64,
}

impl Mov64 {
    pub fn new(reg: Register, value: i64) -> Box<Self> {
        Box::new(Self {
            rex: REX_WRITE,
            opcode: 0xb8 + reg as u8,
            value,
        })
    }
}

impl Instruction for Mov64 {}

#[allow(dead_code)]
#[repr(packed)]
pub struct Mov64rr {
    rex: u8,
    opcode: u8,
    mod_rm: u8,
}

impl Mov64rr {
    pub fn new(dst: Register, src: Register) -> Box<Self> {
        Box::new(Self {
            rex: REX_WRITE,
            opcode: 0x89,
            mod_rm: 3 << 6 | (src as u8) << 3 | dst as u8,
        })
    }
}

impl Instruction for Mov64rr {}

#[allow(dead_code)]
#[repr(packed)]
pub struct Sub64 {
    rex: u8,
    opcode: u8,
    mod_rm: u8,
    value: i32,
}

impl Sub64 {
    pub fn new(reg: Register, value: i32) -> Box<Self> {
        Box::new(Self {
            rex: REX_WRITE,
            opcode: 0x81,
            mod_rm: 3 << 6 | 5 << 3 | (reg as u8),
            value,
        })
    }
}

impl Instruction for Sub64 {}

#[allow(dead_code)]
#[repr(packed)]
pub struct Mov64Ref {
    rex: u8,
    opcode: u8,
    mod_rm: u8,
    offset: i8,
}

impl Mov64Ref {
    pub fn new(dst: Register, src: Register, offset: i8) -> Box<Self> {
        Box::new(Self {
            rex: 0x48,
            opcode: 0x8b,
            mod_rm: 1 << 6 | (src as u8) << 3 | (dst as u8),
            offset,
        })
    }
}

impl Instruction for Mov64Ref {}

#[allow(dead_code)]
#[repr(packed)]
pub struct Div64 {
    rex: u8,
    opcode: u8,
    mod_rm: u8,
}

impl Div64 {
    pub fn new(divider: Register) -> Box<Self> {
        Box::new(Self {
            rex: 0x48,
            opcode: 0xf7,
            mod_rm: 3 << 6 | 6 << 3 | divider as u8,
        })
    }
}

impl Instruction for Div64 {}
