use lazy_static::lazy_static;
use std::{collections::HashMap, fmt::Display};

use self::register::RegisterSize;

const REGISTER_EXT_INDEX: u8 = 0x30;
const JMP_PREFIX: u8 = 0x0F;

const REX_WRITE: u8 = 0b01001000;
const REX_READ: u8 = 0b01000100;
const REX_X: u8 = 0b01000010;
const REX_B: u8 = 0b01000001;

const IMM16_PREFIX: u8 = 0x66;

const MOD_ADDRESS: u8 = 0b00;
const MOD_DISP8: u8 = 0b01;
const MOD_DISP32: u8 = 0b10;
const MOD_REG: u8 = 0b11;

const RM_DISP32: u8 = 0b101;

pub mod register {
    use paste::paste;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum RegisterSize {
        W,
        D,
        Q,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Register {
        pub code: u8,
        pub size: RegisterSize,
        pub ext: bool,
    }

    impl Register {
        const fn new(code: u8, size: RegisterSize, ext: bool) -> Self {
            Self { code, size, ext }
        }
    }

    macro_rules! registers {
    ( $($name:ident = ($attr:literal $(, $ext:ident)? )),* ) => {
        $(
            macro_rules! ext_match {
                () => {
                    pub const $name: Register = Register::new($attr, RegisterSize::W, false);
                    paste! {
                        pub const [<E $name>]: Register = Register::new($attr, RegisterSize::D, false );
                        pub const [<R $name>]: Register = Register::new($attr, RegisterSize::Q, false);
                    }
                };
                ( $($ext:ident)?) => {
                    pub const $name: Register = Register::new($attr, RegisterSize::Q, true);
                    paste! {
                        pub const [<$name D>]: Register = Register::new($attr, RegisterSize::D, true );
                        pub const [<$name W>]: Register = Register::new($attr, RegisterSize::W, true);
                    }
                };
            }
            ext_match!( $($ext:ident)? );

        )*
    };
}

    registers!(
        AX = (0x0),
        CX = (0x1),
        DX = (0x2),
        BX = (0x3),
        SP = (0x4),
        BP = (0x5),
        SI = (0x6),
        DI = (0x7),
        R8 = (0x0, ext),
        R9 = (0x1, ext),
        R10 = (0x2, ext)
    );

    // const AX: Register = Register::new(0x0, RegisterSize::W, false);
    // const EAX: Register = Register::new(0x0, RegisterSize::D, false);
    // const RAX: Register = Register::new(0x0, RegisterSize::Q, false);

    impl From<Register> for u8 {
        fn from(reg: Register) -> Self {
            reg.code
        }
    }

    impl From<Register> for u16 {
        fn from(reg: Register) -> Self {
            reg.code as u16
        }
    }

    impl From<Register> for u32 {
        fn from(reg: Register) -> Self {
            reg.code as u32
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operand {
    None,
    Register(register::Register),
    Imm8(u8),
    Imm16(u16),
    Imm32(u32),
    Imm64(u64),
    Offset8(u8),
    Offset32(i32),
}

impl Operand {
    pub fn is_imm(&self) -> bool {
        match self {
            Operand::Imm16(_) | Operand::Imm32(_) | Operand::Imm64(_) => true,
            _ => false,
        }
    }

    pub fn as_vec(&self) -> Vec<u8> {
        match self {
            Operand::Imm16(value) => value.to_le_bytes().to_vec(),
            Operand::Imm32(value) => value.to_le_bytes().to_vec(),
            Operand::Imm64(value) => value.to_le_bytes().to_vec(),
            Operand::Offset8(value) => value.to_le_bytes().to_vec(),
            Operand::Offset32(value) => value.to_le_bytes().to_vec(),
            _ => unreachable!("Operand {:#?} cannot be converted to vec", &self),
        }
    }
}



impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Imm16(value) => write!(f, "Imm16( 0x{:0x} )", value),
            Operand::Imm32(value) => write!(f, "Imm32( 0x{:0x} )", value),
            Operand::Imm64(value) => write!(f, "Imm64( 0x{:0x} )", value),
            Operand::Offset8(value) => write!(f, "Offset8( 0x{:0x} )", value),
            Operand::Offset32(value) => write!(f, "Offset32( 0x{:0x} )", value),
            _ => write!(f, "{:#?}", self),
        }
    }
}

impl From<register::Register> for Operand {
    fn from(value: register::Register) -> Self {
        Operand::Register(value)
    }
}

impl From<u8> for Operand {
    fn from(value: u8) -> Self {
        Operand::Imm8(value)
    }
}

impl From<u16> for Operand {
    fn from(value: u16) -> Self {
        Operand::Imm16(value)
    }
}

impl From<u32> for Operand {
    fn from(value: u32) -> Self {
        Operand::Imm32(value)
    }
}

impl From<u64> for Operand {
    fn from(value: u64) -> Self {
        Operand::Imm64(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperandEncoding {
    MR,
    RM,
    MI,
    OI,
    M,
    I,
    D,
}

#[derive(Debug, Clone)]
pub struct Mnemonic {
    name: MnemonicName,
    has_rex_w: bool,
    has_jump_prefix: bool,
    reg: u8,
    rm: u8,
    opcodes: HashMap<OperandEncoding, u8>,
    op1: Operand,
    op2: Operand,
    disp: Operand,
    symbol: Option<String>,
    value_loc: usize,
}

impl Display for Mnemonic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let opcodes: String = self.opcodes.iter().fold(String::new(), |acc, (k, v)| {
            acc + &format!("{:#?}: 0x{:0x}, ", *k, *v)
        });
        write!(
            f,
            "{:#?} {{
            has_rex_w: {},
            reg: {},
            rm: {},
            opcodes:{{ {opcodes} }},
            op1: {},
            op2: {},
            disp: {},
            symbol: {:?},
            value_loc: {},
        }}",
            self.name,
            self.has_rex_w,
            self.reg,
            self.rm,
            self.op1,
            self.op2,
            self.disp,
            self.symbol,
            self.value_loc,
        )
    }
}

impl Mnemonic {
    fn new(name: MnemonicName) -> Self {
        Mnemonic {
            name,
            has_rex_w: true,
            has_jump_prefix: false,
            opcodes: HashMap::new(),
            reg: 0,
            rm: 0,
            op1: Operand::None,
            op2: Operand::None,
            disp: Operand::None,
            symbol: None,
            value_loc: 0,
        }
    }

    pub fn get_name(&self) -> MnemonicName {
        self.name
    }

    pub fn set_op1(&mut self, op: impl Into<Operand>) {
        self.op1 = op.into();
    }

    pub fn set_op2(&mut self, op: impl Into<Operand>) {
        self.op2 = op.into();
    }

    pub fn get_value_loc(&self) -> usize {
        let mut cloned = self.clone();
        cloned.as_vec();
        cloned.value_loc
    }

    pub fn get_symbol(&self) -> Option<&str> {
        assert_eq!(self.name, MnemonicName::Call);
        self.symbol.as_deref()
    }

    pub fn opcode(&self, opcode: u8, operands: OperandEncoding) -> Self {
        let mut cloned = self.clone();
        cloned.opcodes.insert(operands, opcode);
        cloned
    }

    pub fn no_rex_w(&self) -> Self {
        let mut cloned = self.clone();
        cloned.has_rex_w = false;
        cloned
    }

    pub fn has_jump_prefix(&self) -> Self {
        let mut cloned = self.clone();
        cloned.has_jump_prefix = true;
        cloned
    }

    pub fn reg(&self, reg: u8) -> Self {
        let mut cloned = self.clone();
        cloned.reg = reg;
        cloned
    }

    pub fn rm(&self, rm: u8) -> Self {
        let mut cloned = self.clone();
        cloned.rm = rm;
        cloned
    }

    pub fn op1(&self, op: impl Into<Operand>) -> Self {
        if self.op1 != Operand::None {
            panic!("op1 is already assigned")
        }
        let mut cloned = self.clone();
        cloned.op1 = op.into();
        cloned
    }

    pub fn op2(&self, op: impl Into<Operand>) -> Self {
        if self.op2 != Operand::None {
            panic!("op2 is already assigned")
        }
        let mut cloned = self.clone();
        cloned.op2 = op.into();
        cloned
    }

    pub fn disp(&self, op: impl Into<Operand>) -> Self {
        if self.disp != Operand::None {
            panic!("disp is already assigned")
        }
        let mut cloned = self.clone();
        cloned.disp = op.into();
        cloned
    }

    pub fn symbol(&mut self, symbol: String) -> Self {
        assert_eq!(self.name, MnemonicName::Call);
        let mut cloned = self.clone();
        cloned.symbol = Some(symbol);
        cloned
    }

    pub fn as_vec(&mut self) -> Vec<u8> {
        let mut result = vec![];
        let mut operand_enc;

        let mut prefix = 0;
        let mut _mod = 0;
        let mut reg = self.reg;
        let mut rm = self.rm;

        match self.op1 {
            Operand::Register(dst) => {
                match dst.size {
                    RegisterSize::W => prefix |= IMM16_PREFIX,
                    RegisterSize::D => (),
                    RegisterSize::Q => {
                        if self.has_rex_w {
                            prefix |= REX_WRITE;
                        }
                    }
                }
                _mod = MOD_REG;
                rm = dst.code;

                if dst.ext {
                    prefix |= REX_B;
                }

                match self.op2 {
                    Operand::Register(src) => {
                        assert!(
                            src.size == dst.size,
                            "Register sizes do not match: {:#?}, {:#?}",
                            src,
                            dst
                        );
                        operand_enc = Some(OperandEncoding::RM);

                        _mod = match self.disp {
                            Operand::Offset8(_) => MOD_DISP8,
                            Operand::Offset32(_) => MOD_DISP32,
                            _ => {
                                operand_enc = Some(OperandEncoding::MR);
                                MOD_REG
                            }
                        };

                        reg = src.code;
                        if src.ext {
                            prefix |= REX_READ;
                        }
                    }
                    _ => {
                        if let Some(oi_opcode) = self.opcodes.get_mut(&OperandEncoding::OI) {
                            *oi_opcode += dst.code;
                            operand_enc = Some(OperandEncoding::OI);
                        } else if self.opcodes.get_mut(&OperandEncoding::M).is_some()  {
                            operand_enc = Some(OperandEncoding::M);
                        } else {
                            operand_enc = Some(OperandEncoding::MI);
                        }
                    }
                }
            }
            Operand::Imm16(_) => {
                prefix |= IMM16_PREFIX;
                operand_enc = Some(OperandEncoding::I)
            }
            Operand::Imm32(_) => operand_enc = Some(OperandEncoding::I),
            Operand::Imm64(_) => operand_enc = Some(OperandEncoding::I),
            Operand::Offset32(_) => operand_enc = Some(OperandEncoding::D),
            _ => unreachable!("Invalid first operand: {:#?}", self.op1),
        }
        if prefix != 0 {
            result.extend(prefix.to_le_bytes());
        }
        if self.has_jump_prefix {
            result.extend(JMP_PREFIX.to_le_bytes());
        }

        // dbg!(operand_enc);
        // println!("{}", &self);
        let opcode = self
            .opcodes
            .get(&operand_enc.unwrap())
            .unwrap_or_else(|| panic!("No encoding for operands: {:?}, {:?}", self.op1, self.op2));
        result.extend(opcode.to_le_bytes());

        let mod_rm = _mod << 6 | reg << 3 | rm;
        // println!("{:0b}", mod_rm);

        match operand_enc.unwrap() {
            OperandEncoding::MR => {
                result.extend(mod_rm.to_le_bytes());
            }
            OperandEncoding::RM => {
                let mod_rm = _mod << 6 | rm << 3 | reg;
                result.extend(mod_rm.to_le_bytes());
                self.value_loc = result.len();
                match self.disp {
                    Operand::Offset8(_) | Operand::Offset32(_) => result.extend(self.disp.as_vec()),
                    _ => (),
                }
            }
            OperandEncoding::M => {
                _mod = match self.disp {
                    Operand::Offset8(_) => MOD_DISP8,
                    Operand::Offset32(_) => MOD_DISP32,
                    _ => {
                        MOD_REG
                    }
                };
                let mod_rm = _mod << 6 | reg << 3 | rm;
                result.extend(mod_rm.to_le_bytes());
                self.value_loc = result.len();
                match self.disp {
                    Operand::Offset8(_) | Operand::Offset32(_) => result.extend(self.disp.as_vec()),
                    _ => (),
                }
            }
            OperandEncoding::MI => {
                _mod = match self.disp {
                    Operand::Offset8(_) => MOD_DISP8,
                    Operand::Offset32(_) => MOD_DISP32,
                    _ => {
                        MOD_REG
                    }
                };
                let mod_rm = _mod << 6 | reg << 3 | rm;
                result.extend(mod_rm.to_le_bytes());
                if let Operand::Offset8(_) | Operand::Offset32(_) = self.disp {
                    result.extend(self.disp.as_vec())
                }
                if self.op2.is_imm() {
                    self.value_loc = result.len();
                    result.extend(self.op2.as_vec());
                }
            }
            OperandEncoding::OI => {
                if self.op2.is_imm() {
                    self.value_loc = result.len();
                    result.extend(self.op2.as_vec());
                }
            }
            OperandEncoding::I => {
                if mod_rm != 0 {
                    result.extend(mod_rm.to_le_bytes());
                }
                self.value_loc = result.len();
                result.extend(self.op1.as_vec());
            }
            OperandEncoding::D => {
                self.value_loc = result.len();
                match self.op1 {
                    Operand::Offset32(value) => {
                        let mut value = value;
                        if value < 0 {
                            value = !(-value) + 1;
                        }
                        // println!("{value:0x}");
                        result.extend(value.to_le_bytes().to_vec());
                    }
                    _ => panic!("Invalid operand {} for encoding D", self.op1),
                };
            }
        }

        result
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MnemonicName {
    Mov,
    Add,
    Sub,
    Div,
    Inc,
    Xor,
    Push,
    Pop,
    Call,
    Cmp,
    Jmp,
    Jl,
    Jle,
    Jg,
    Jge,
    Je,
    Jz,
}

lazy_static! {
    pub static ref MOV: Mnemonic = Mnemonic::new(MnemonicName::Mov)
        .opcode(0x89, OperandEncoding::MR)
        .opcode(0x8B, OperandEncoding::RM)
        .opcode(0xB8, OperandEncoding::OI);
    pub static ref ADD: Mnemonic = Mnemonic::new(MnemonicName::Add)
        .opcode(0x01, OperandEncoding::MR)
        .opcode(0x81, OperandEncoding::MI);
    pub static ref SUB: Mnemonic = Mnemonic::new(MnemonicName::Sub)
        .opcode(0x29, OperandEncoding::MR)
        .opcode(0x81, OperandEncoding::MI)
        .reg(5);
    pub static ref DIV: Mnemonic = Mnemonic::new(MnemonicName::Div)
        .opcode(0xF7, OperandEncoding::MI)
        .reg(6);
    pub static ref INC: Mnemonic = Mnemonic::new(MnemonicName::Inc)
        .opcode(0xFF, OperandEncoding::M)
        .reg(0);
    pub static ref XOR: Mnemonic = Mnemonic::new(MnemonicName::Xor)
        .opcode(0x31, OperandEncoding::MR)
        .opcode(0x81, OperandEncoding::MI).reg(6);

    pub static ref PUSH: Mnemonic = Mnemonic::new(MnemonicName::Push)
        .opcode(0x50, OperandEncoding::OI)
        .opcode(0x68, OperandEncoding::I)
        .no_rex_w();
    pub static ref POP: Mnemonic = Mnemonic::new(MnemonicName::Pop)
        .opcode(0x58, OperandEncoding::OI)
        .no_rex_w();
    pub static ref CALL: Mnemonic = Mnemonic::new(MnemonicName::Call)
        .opcode(0xE8, OperandEncoding::D)
        .no_rex_w();
    pub static ref CMP: Mnemonic = Mnemonic::new(MnemonicName::Cmp)
        .opcode(0x81, OperandEncoding::MI)
        .reg(7);
    pub static ref JMP: Mnemonic = Mnemonic::new(MnemonicName::Jmp)
        .opcode(0xFF, OperandEncoding::I)
        .reg(4)
        .rm(RM_DISP32)
        .no_rex_w();
    pub static ref JLE: Mnemonic = Mnemonic::new(MnemonicName::Jle)
        .opcode(0x8E, OperandEncoding::D)
        .rm(RM_DISP32)
        .no_rex_w()
        .has_jump_prefix();
    pub static ref JL: Mnemonic = Mnemonic::new(MnemonicName::Jl)
        .opcode(0x8C, OperandEncoding::D)
        .rm(RM_DISP32)
        .no_rex_w()
        .has_jump_prefix();

}

lazy_static! {
    pub static ref SIZE_OF_JMP: usize = JMP.op1(Operand::Imm32(0)).as_vec().len();
}

#[cfg(test)]
mod tests {
    use crate::emitter::mnemonics::*;
    use rstest::*;

    #[rstest]
    #[case::imm16(register::CX, 0xABCD_u16, vec ! [0x66, 0xB9, 0xCD, 0xAB])]
    #[case::imm32(register::ECX, 0xABCDEF12_u32, vec ! [ 0xB9, 0x12, 0xEF, 0xCD, 0xAB])]
    #[case::imm64(
        register::RCX, 0xABCDEF12ABCDEF12_u64, vec ! [ 0x48, 0xB9, 0x12, 0xEF, 0xCD, 0xAB, 0x12, 0xEF, 0xCD, 0xAB]
    )]
    #[case::CxAx(register::CX, register::AX, vec ! [0x66, 0x89, 0xC1])]
    #[case::EcxEax(register::ECX, register::EAX, vec ! [0x89, 0xC1])]
    #[case::RcxRax(register::RCX, register::RAX, vec ! [0x48, 0x89, 0xC1])]
    fn test_mov(
        #[case] op1: impl Into<Operand>,
        #[case] op2: impl Into<Operand>,
        #[case] expected: Vec<u8>,
    ) {
        let mut instruction = MOV.op1(op1).op2(op2);
        assert_eq!(instruction.as_vec(), expected);
    }

    #[rstest]
    #[case::imm64(register::R8D, 0xABCDEF12_u32, vec ! [0x41, 0xB8, 0x12, 0xEF, 0xCD, 0xAB])]
    #[case::R8Rax(register::R8, register::RAX, vec ! [0x49, 0x89, 0xC0])]
    #[case::R8R9(register::R8, register::R9, vec ! [0x4D, 0x89, 0xC8])]
    fn test_mov_ext(
        #[case] op1: impl Into<Operand>,
        #[case] op2: impl Into<Operand>,
        #[case] expected: Vec<u8>,
    ) {
        let mut instruction = MOV.op1(op1).op2(op2);
        assert_eq!(instruction.as_vec(), expected);
    }

    #[rstest]
    #[case::RcxRax8(
        register::RCX, register::RAX, Operand::Offset8(0xAB), vec ! [0x48, 0x8B, 0x48, 0xAB]
    )]
    #[case::RcxRax32(
        register::RCX, register::RAX, Operand::Offset32(0xABCDEF1_i32), vec ! [0x48, 0x8B, 0x88, 0xF1, 0xDE, 0xBC, 0x0A]
    )]
    fn test_mov_offset(
        #[case] op1: impl Into<Operand>,
        #[case] op2: impl Into<Operand>,
        #[case] op3: Operand,
        #[case] expected: Vec<u8>,
    ) {
        let mut instruction = MOV.op1(op1).op2(op2).disp(op3);
        assert_eq!(instruction.as_vec(), expected);
    }

    #[rstest]
    #[case::imm16(0xABCD_u16, vec ! [0x66, 0x68, 0xCD, 0xAB])]
    #[case::imm32(0xABCDEF12_u32, vec ! [0x68, 0x12, 0xEF, 0xCD, 0xAB])]
    #[case::Rcx(register::RCX, vec ! [0x51])]
    fn test_push(#[case] op1: impl Into<Operand>, #[case] expected: Vec<u8>) {
        let mut instruction = PUSH.op1(op1);
        assert_eq!(instruction.as_vec(), expected);
    }

    #[rstest]
    #[case::Rcx(register::ECX, vec ! [0x59])]
    #[case::R9(register::R9, vec ! [0x41, 0x59])]
    fn test_pop(#[case] op1: impl Into<Operand>, #[case] expected: Vec<u8>) {
        let mut instruction = POP.op1(op1);
        assert_eq!(instruction.as_vec(), expected);
    }

    #[rstest]
    #[case::imm32(register::ECX, 0xABCDEF12_u32, vec ! [0x81, 0xC1, 0x12, 0xEF, 0xCD, 0xAB])]
    #[case::Rimm32(register::RCX, 0xABCDEF12_u32, vec ! [0x48, 0x81, 0xC1, 0x12, 0xEF, 0xCD, 0xAB])]
    #[case::RcxRax(register::RCX, register::RAX, vec ! [0x48, 0x01, 0xC1])]
    fn test_add(
        #[case] op1: impl Into<Operand>,
        #[case] op2: impl Into<Operand>,
        #[case] expected: Vec<u8>,
    ) {
        let mut instruction = ADD.op1(op1).op2(op2);
        assert_eq!(instruction.as_vec(), expected);
    }

    #[rstest]
    #[case::imm32(register::ECX, 0xABCDEF12_u32, vec ! [0x81, 0xE9, 0x12, 0xEF, 0xCD, 0xAB])]
    #[case::Rimm32(register::RCX, 0xABCDEF12_u32, vec ! [0x48, 0x81, 0xE9, 0x12, 0xEF, 0xCD, 0xAB])]
    #[case::RcxRax(register::RCX, register::RAX, vec ! [0x48, 0x29, 0xC1])]
    fn test_sub(
        #[case] op1: impl Into<Operand>,
        #[case] op2: impl Into<Operand>,
        #[case] expected: Vec<u8>,
    ) {
        let mut instruction = SUB.op1(op1).op2(op2);
        assert_eq!(instruction.as_vec(), expected);
    }

    #[rstest]
    #[case::Eax(register::ECX, vec ! [0xF7, 0xF1])]
    #[case::Rcx(register::RCX, vec ! [0x48, 0xF7, 0xF1])]
    fn test_div(#[case] op1: impl Into<Operand>, #[case] expected: Vec<u8>) {
        let mut instruction = DIV.op1(op1);
        assert_eq!(instruction.as_vec(), expected);
    }

    #[rstest]
    #[case::Rcx(register::RCX, vec ! [0x48, 0xFF, 0xC1])]
    fn test_inc(#[case] op1: impl Into<Operand>, #[case] expected: Vec<u8>) {
        let mut instruction = INC.op1(op1);
        assert_eq!(instruction.as_vec(), expected);
    }

    #[rstest]
    #[case::Rcx(register::RCX, Operand::Offset32(0xFFFF), vec ! [0x48, 0xFF, 0x81, 0xFF, 0xFF, 0x0, 0x0])]
    fn test_inc_offset(#[case] op1: impl Into<Operand>, #[case] disp: impl Into<Operand>, #[case] expected: Vec<u8>) {
        let mut instruction = INC.op1(op1).disp(disp);
        assert_eq!(instruction.as_vec(), expected);
    }


    #[rstest]
    #[case::Offset32(Operand::Offset32(0xABCDEF1), vec ! [0xE8, 0xF1, 0xDE, 0xBC, 0x0A])]
    fn test_call(#[case] op1: impl Into<Operand>, #[case] expected: Vec<u8>) {
        let mut instruction = CALL.op1(op1);
        assert_eq!(instruction.as_vec(), expected);
    }

    #[rstest]
    #[case::imm16(register::CX, 0xABCD_u16, vec ! [0x66, 0x81, 0xF9, 0xCD, 0xAB])]
    #[case::imm32(register::ECX, 0xABCDEF12_u32, vec ! [ 0x81, 0xF9, 0x12, 0xEF, 0xCD, 0xAB])]
    #[case::imm64(
        register::RCX, 0xABCDEF12ABCDEF12_u64, vec ! [ 0x48, 0x81, 0xF9, 0x12, 0xEF, 0xCD, 0xAB, 0x12, 0xEF, 0xCD, 0xAB]
    )]
    fn test_cmp(
        #[case] op1: impl Into<Operand>,
        #[case] op2: impl Into<Operand>,
        #[case] expected: Vec<u8>,
    ) {
        let mut instruction = CMP.op1(op1).op2(op2);
        assert_eq!(instruction.as_vec(), expected);
    }

    #[rstest]
    #[case::Imm64(0xABCDEF12_u64, vec ! [0xFF, 0x25, 0x12, 0xEF, 0xCD, 0xAB, 0x0, 0x0, 0x0, 0x0])]
    fn test_jmp(#[case] op1: impl Into<Operand>, #[case] expected: Vec<u8>) {
        let mut instruction = JMP.op1(op1);
        assert_eq!(instruction.as_vec(), expected);
    }

    #[rstest]
    #[case::Offset32(Operand::Offset32(-0xDC), vec ![0x0F, 0x8E, 0x24, 0xff, 0xff, 0xff])]
    #[case::Offset32(Operand::Offset32(-0xABCDEF1), vec![0x0F, 0x8E ,0x0f,0x21,0x43 , 0xf5 ])]
    fn test_jle(#[case] op1: impl Into<Operand>, #[case] expected: Vec<u8>) {
        let mut instruction = JLE.op1(op1);
        assert_eq!(instruction.as_vec(), expected);
    }

    #[rstest]
    #[should_panic]
    fn test_invalid_encoding() {
        let mut instruction = MOV.op1(Operand::Offset32(0)).op2(Operand::Offset32(0));
        instruction.as_vec();
    }
}
