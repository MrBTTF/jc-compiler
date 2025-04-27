use crate::emitter::data::Data;
use crate::emitter::data::DataLocation;
use crate::emitter::stack::StackManager;

use super::super::{code_context::CodeContext, mnemonics::*};

pub const STDOUT_FD: u64 = 0x1;

pub const SYS_WRITE: u64 = 0x1;
pub const SYS_EXIT: u64 = 0x3c;

pub const ARG_REGISTERS: &[register::Register] = &[
    register::RDI,
    register::RSI,
    register::RDX,
    register::RCX,
    register::R8,
    register::R9,
];

pub fn push_args(code_context: &mut CodeContext, stack: &mut StackManager, args: &[Data]) {
    args.iter().enumerate().for_each(|(i, _)| {
        code_context.add_slice(&stack.push_register(ARG_REGISTERS[i]));
    });

    args.iter()
        .enumerate()
        .for_each(|(i, arg)| match &arg.data_loc {
            DataLocation::Stack(stack_loc) => {
                dbg!(&arg);
                let data_loc: u32 = stack_loc.into();
                code_context.add_slice(&[
                    MOV.op1(ARG_REGISTERS[i]).op2(register::RBP),
                    SUB.op1(ARG_REGISTERS[i]).op2(data_loc),
                ]);
                if !arg.reference {
                    code_context.add(
                        MOV.op1(ARG_REGISTERS[i])
                            .op2(ARG_REGISTERS[i])
                            .disp(Operand::Offset32(0)),
                    );
                }
            }
            DataLocation::DataSection(_) => {
                code_context.add(
                    MOV.op1(ARG_REGISTERS[i])
                        .op2(0_u64)
                        .symbol(arg.symbol.clone()),
                );

                if !arg.reference {
                    code_context.add(
                        MOV.op1(ARG_REGISTERS[i])
                            .op2(ARG_REGISTERS[i])
                            .disp(Operand::Offset32(0)),
                    );
                }
            }
        });
}

pub fn pop_args(code_context: &mut CodeContext, stack: &mut StackManager, args_count: usize) {
    (0..args_count).rev().for_each(|i| {
        code_context.add_slice(&stack.pop_register(ARG_REGISTERS[i]));
    });
}
