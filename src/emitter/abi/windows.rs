use super::super::{mnemonics::*, structs::Instructions};

#[derive(Debug)]
pub enum Arg {
    Data(i64),
    Stack(i64),
}

const ARG_REGISTERS: &[register::Register] =
    &[register::RCX, register::RDX, register::R8, register::R9];

pub fn push_args(args: &[Arg]) -> Instructions {
    args.iter()
        .enumerate()
        .fold(vec![], |mut acc: Instructions, (i, arg)| {
            acc.push(PUSH.op1(Operand::Register(ARG_REGISTERS[i])));
            match arg {
                Arg::Stack(loc) => {
                    acc.push(
                        MOV.op1(Operand::Register(ARG_REGISTERS[i]))
                            .op2(Operand::Register(register::RBP)),
                    );
                    acc.push(
                        SUB.op1(Operand::Register(ARG_REGISTERS[i]))
                            .op2(Operand::Imm32(*loc as u32)),
                    );
                }
                Arg::Data(loc) => acc.push(
                    MOV.op1(Operand::Register(ARG_REGISTERS[i]))
                        .op2(Operand::Imm64(*loc as u64)),
                ),
            }
            acc
        })
}

pub fn pop_args(args_count: usize) -> Instructions {
    (0..args_count).fold(vec![], |mut acc: Instructions, i| {
        acc.push(POP.op1(Operand::Register(ARG_REGISTERS[i])));
        acc
    })
}
