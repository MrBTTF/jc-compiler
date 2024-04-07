use lazy_static::lazy_static;
use std::{collections::HashMap, fmt::Display};

use self::register::RegisterSize;

const REGISTER_EXT_INDEX: u8 = 0x30;

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
        R9 = (0x1, ext)
    );

    // const AX: Register = Register::new(0x0, RegisterSize::W, false);
    // const EAX: Register = Register::new(0x0, RegisterSize::D, false);
    // const RAX: Register = Register::new(0x0, RegisterSize::Q, false);

    impl From<Register> for u8 {
        fn from(reg: Register) -> Self {
            reg.code as u8
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
    Imm8(u16),
    Imm16(u16),
    Imm32(u32),
    Imm64(u64),
    Offset8(u8),
    Offset32(u32),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperandEncoding {
    MR,
    RM,
    MI,
    OI,
    I,
    D,
}

#[derive(Debug, Clone)]
pub struct Mnemonic {
    name: String,
    has_rex_w: bool,
    reg: u8,
    rm: u8,
    opcodes: HashMap<OperandEncoding, u8>,
    op1: Operand,
    op2: Operand,
    op3: Operand,
    value_loc: usize,
}

impl Display for Mnemonic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let opcodes: String = self.opcodes.iter().fold(String::new(), |acc, (k, v)| {
            acc + &format!("{:#?}: 0x{:0x}, ", *k, *v)
        });
        write!(
            f,
            "{} {{
            has_rex_w: {},
            reg: {},
            rm: {},
            opcodes:{{ {opcodes} }},
            op1: {},
            op2: {},
            op3: {},
            value_loc: {},
        }}",
            self.name,
            self.has_rex_w,
            self.reg,
            self.rm,
            self.op1,
            self.op2,
            self.op3,
            self.value_loc,
        )
    }
}

impl Mnemonic {
    fn new(name: String) -> Self {
        Mnemonic {
            name,
            has_rex_w: true,
            opcodes: HashMap::new(),
            reg: 0,
            rm: 0,
            op1: Operand::None,
            op2: Operand::None,
            op3: Operand::None,
            value_loc: 0,
        }
    }

    pub fn set_op1(&mut self, op: Operand) {
        self.op1 = op;
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

    pub fn op1(&self, op: Operand) -> Self {
        if self.op1 != Operand::None {
            panic!("op1 is already assigned")
        }
        let mut cloned = self.clone();
        cloned.op1 = op;
        cloned
    }

    pub fn op2(&self, op: Operand) -> Self {
        if self.op2 != Operand::None {
            panic!("op2 is already assigned")
        }
        let mut cloned = self.clone();
        cloned.op2 = op;
        cloned
    }

    pub fn op3(&self, op: Operand) -> Self {
        if self.op3 != Operand::None {
            panic!("op3 is already assigned")
        }
        let mut cloned = self.clone();
        cloned.op3 = op;
        cloned
    }

    pub fn as_vec(&mut self) -> Vec<u8> {
        let mut result = vec![];
        let mut operand_enc = None;

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

                        _mod = match self.op3 {
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
        // dbg!(operand_enc);
        // println!("{}", &self);
        let opcode = self.opcodes.get(&operand_enc.unwrap()).unwrap();
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
                match self.op3 {
                    Operand::Offset8(_) | Operand::Offset32(_) => result.extend(self.op3.as_vec()),
                    _ => (),
                }
            }
            OperandEncoding::MI => {
                result.extend(mod_rm.to_le_bytes());
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
                result.extend(self.op1.as_vec());
            }
        }

        result
    }
}

lazy_static! {
    pub static ref MOV: Mnemonic = Mnemonic::new("MOV".to_string())
        .opcode(0x89, OperandEncoding::MR)
        .opcode(0x8B, OperandEncoding::RM)
        .opcode(0xB8, OperandEncoding::OI);
    pub static ref ADD: Mnemonic = Mnemonic::new("ADD".to_string())
        .opcode(0x01, OperandEncoding::MR)
        .opcode(0x81, OperandEncoding::MI);
    pub static ref SUB: Mnemonic = Mnemonic::new("SUB".to_string())
        .opcode(0x29, OperandEncoding::MR)
        .opcode(0x81, OperandEncoding::MI)
        .reg(5);
    pub static ref DIV: Mnemonic = Mnemonic::new("DIV".to_string())
        .opcode(0xF7, OperandEncoding::MI)
        .reg(6);
    pub static ref PUSH: Mnemonic = Mnemonic::new("PUSH".to_string())
        .opcode(0x50, OperandEncoding::OI)
        .opcode(0x68, OperandEncoding::I)
        .no_rex_w();
    pub static ref POP: Mnemonic = Mnemonic::new("POP".to_string())
        .opcode(0x58, OperandEncoding::OI)
        .no_rex_w();
    pub static ref CALL: Mnemonic = Mnemonic::new("CALL".to_string())
        .opcode(0xE8, OperandEncoding::D)
        .no_rex_w();
    pub static ref JMP: Mnemonic = Mnemonic::new("JMP".to_string())
        .opcode(0xFF, OperandEncoding::I)
        .reg(4)
        .rm(RM_DISP32)
        .no_rex_w();
}

#[cfg(test)]
mod tests {
    use crate::emitter::mnemonics::*;
    use rstest::*;

    #[rstest]
    #[case::imm16(Operand::Register(register::CX), Operand::Imm16(0xABCD), vec![0x66, 0xB9, 0xCD, 0xAB])]
    #[case::imm32(Operand::Register(register::ECX), Operand::Imm32(0xABCDEF12), vec![ 0xB9, 0x12, 0xEF, 0xCD, 0xAB])]
    #[case::imm64(Operand::Register(register::RCX), Operand::Imm64(0xABCDEF12ABCDEF12), vec![ 0x48,0xB9,0x12,0xEF,0xCD,0xAB,0x12,0xEF,0xCD,0xAB])]
    #[case::CxAx(Operand::Register(register::CX), Operand::Register(register::AX), vec![0x66,0x89,0xC1])]
    #[case::EcxEax(Operand::Register(register::ECX), Operand::Register(register::EAX), vec![0x89,0xC1])]
    #[case::RcxRax(Operand::Register(register::RCX), Operand::Register(register::RAX), vec![0x48,0x89,0xC1])]
    fn test_mov(#[case] op1: Operand, #[case] op2: Operand, #[case] expected: Vec<u8>) {
        let mut instruction = MOV.op1(op1).op2(op2);
        assert_eq!(instruction.as_vec(), expected);
    }

    #[rstest]
    #[case::imm64(Operand::Register(register::R8D), Operand::Imm32(0xABCDEF12), vec![0x41,0xB8,0x12,0xEF,0xCD,0xAB])]
    #[case::R8Rax(Operand::Register(register::R8), Operand::Register(register::RAX), vec![0x49,0x89,0xC0])]
    #[case::R8R9(Operand::Register(register::R8), Operand::Register(register::R9), vec![0x4D,0x89,0xC8])]
    fn test_mov_ext(#[case] op1: Operand, #[case] op2: Operand, #[case] expected: Vec<u8>) {
        let mut instruction = MOV.op1(op1).op2(op2);
        assert_eq!(instruction.as_vec(), expected);
    }

    #[rstest]
    #[case::RcxRax8(Operand::Register(register::RCX), Operand::Register(register::RAX),Operand::Offset8(0xAB), vec![0x48,0x8B,0x48,0xAB])]
    #[case::RcxRax32(Operand::Register(register::RCX), Operand::Register(register::RAX),Operand::Offset32(0xABCDEF12), vec![0x48,0x8B,0x88,0x12,0xEF,0xCD,0xAB])]
    fn test_mov_offset(
        #[case] op1: Operand,
        #[case] op2: Operand,
        #[case] op3: Operand,
        #[case] expected: Vec<u8>,
    ) {
        let mut instruction = MOV.op1(op1).op2(op2).op3(op3);
        assert_eq!(instruction.as_vec(), expected);
    }

    #[rstest]
    #[case::imm16( Operand::Imm16(0xABCD), vec![0x66,0x68,0xCD,0xAB])]
    #[case::imm32( Operand::Imm32(0xABCDEF12), vec![0x68,0x12,0xEF,0xCD,0xAB])]
    #[case::Rcx(Operand::Register(register::RCX),vec![0x51])]
    fn test_push(#[case] op1: Operand, #[case] expected: Vec<u8>) {
        let mut instruction = PUSH.op1(op1);
        assert_eq!(instruction.as_vec(), expected);
    }

    #[rstest]
    #[case::Rcx(Operand::Register(register::ECX),vec![0x59])]
    #[case::R9(Operand::Register(register::R9),vec![0x41, 0x59])]
    fn test_pop(#[case] op1: Operand, #[case] expected: Vec<u8>) {
        let mut instruction = POP.op1(op1);
        assert_eq!(instruction.as_vec(), expected);
    }

    #[rstest]
    #[case::imm32(Operand::Register(register::ECX), Operand::Imm32(0xABCDEF12), vec![0x81,0xC1,0x12,0xEF,0xCD,0xAB])]
    #[case::Rimm32(Operand::Register(register::RCX), Operand::Imm32(0xABCDEF12), vec![0x48,0x81,0xC1,0x12,0xEF,0xCD,0xAB])]
    #[case::RcxRax(Operand::Register(register::RCX), Operand::Register(register::RAX), vec![0x48,0x01,0xC1])]
    fn test_add(#[case] op1: Operand, #[case] op2: Operand, #[case] expected: Vec<u8>) {
        let mut instruction = ADD.op1(op1).op2(op2);
        assert_eq!(instruction.as_vec(), expected);
    }

    #[rstest]
    #[case::imm32(Operand::Register(register::ECX), Operand::Imm32(0xABCDEF12), vec![0x81,0xE9,0x12,0xEF,0xCD,0xAB])]
    #[case::Rimm32(Operand::Register(register::RCX), Operand::Imm32(0xABCDEF12), vec![0x48,0x81,0xE9,0x12,0xEF,0xCD,0xAB])]
    #[case::RcxRax(Operand::Register(register::RCX), Operand::Register(register::RAX), vec![0x48,0x29,0xC1])]
    fn test_sub(#[case] op1: Operand, #[case] op2: Operand, #[case] expected: Vec<u8>) {
        let mut instruction = SUB.op1(op1).op2(op2);
        assert_eq!(instruction.as_vec(), expected);
    }

    #[rstest]
    #[case::Eax(Operand::Register(register::ECX), vec![0xF7, 0xF1])]
    #[case::Rcx(Operand::Register(register::RCX), vec![0x48,0xF7, 0xF1])]
    fn test_div(#[case] op1: Operand, #[case] expected: Vec<u8>) {
        let mut instruction = DIV.op1(op1);
        assert_eq!(instruction.as_vec(), expected);
    }

    #[rstest]
    #[case::Offset32(Operand::Offset32(0xABCDEF12), vec![0xE8,0x12,0xEF,0xCD,0xAB])]
    fn test_call(#[case] op1: Operand, #[case] expected: Vec<u8>) {
        let mut instruction = CALL.op1(op1);
        assert_eq!(instruction.as_vec(), expected);
    }

    #[rstest]
    #[case::Imm64(Operand::Imm64(0xABCDEF12), vec![0xFF, 0x25,0x12,0xEF,0xCD,0xAB, 0x0, 0x0, 0x0, 0x0])]
    fn test_jmp(#[case] op1: Operand, #[case] expected: Vec<u8>) {
        let mut instruction = JMP.op1(op1);
        assert_eq!(instruction.as_vec(), expected);
    }
}
